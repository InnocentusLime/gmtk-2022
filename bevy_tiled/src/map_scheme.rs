use anyhow::{anyhow, bail, ensure, Context};
use bevy_ecs_tilemap::{tiles::{TileBundle, TilePos, TileTexture, TileStorage}, prelude::{TilemapId, TilemapSize, TilemapTexture, TilemapMeshType, TilemapTileSize, TilemapGridSize}, TilemapBundle};
use std::collections::HashMap;

use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::de::DeserializeOwned;
use tiled::{Tileset, Map, Layer, LayerType, TileLayer, FiniteTileLayer, LayerTileData};

use crate::{TiledTileExt, TilesetIndexing, TiledLayerTileExt};

pub trait TileBuilderProvider {
    fn process_tileset(&mut self, set_id: usize, tileset: &Tileset, indexing: &TilesetIndexing) -> anyhow::Result<()>;
    fn run_builder(&mut self, set_id: usize, id: u32, cmds: &mut EntityCommands) -> anyhow::Result<()>;
    fn tilemap_post_build(&mut self, cmds: &mut EntityCommands) -> anyhow::Result<()>;
}

struct TileBuilderProviderImpl<T, F, G, H> 
where
    F: FnMut(&T, &mut EntityCommands) -> anyhow::Result<()>,
    G: FnMut(&T, &Tileset, &TilesetIndexing) -> anyhow::Result<()>,
    H: FnMut(&mut EntityCommands) -> anyhow::Result<()>,
{
    deserialized_props: HashMap<(usize, u32), T>,
    tileset_callback: G,
    tile_builder: F,
    tilemap_post: H,
}

impl<T, F, G, H> TileBuilderProvider for TileBuilderProviderImpl<T, F, G, H> 
where
    T: DeserializeOwned,
    F: FnMut(&T, &mut EntityCommands) -> anyhow::Result<()>,
    G: FnMut(&T, &Tileset, &TilesetIndexing) -> anyhow::Result<()>,
    H: FnMut(&mut EntityCommands) -> anyhow::Result<()>,
{
    fn process_tileset(
        &mut self, 
        set_id: usize, 
        tileset: &Tileset, 
        indexing: &TilesetIndexing
    ) -> anyhow::Result<()> {
        self.deserialized_props.reserve(tileset.tilecount as usize);

        for (id, tile) in tileset.tiles() {
            let props = tile.properties()?;

            (self.tileset_callback)(&props, tileset, indexing);
            self.deserialized_props.insert((set_id, id), props);
        }

        Ok(())
    }

    fn run_builder(&mut self, set_id: usize, id: u32, cmds: &mut EntityCommands) -> anyhow::Result<()> {
        let props = self.deserialized_props.get(&(set_id, id))
            .ok_or_else(|| anyhow!("Tile {} didn't have any deserialized properties", id))?;
        (self.tile_builder)(props, cmds)
    }

    fn tilemap_post_build(&mut self, cmds: &mut EntityCommands) -> anyhow::Result<()> {
        (self.tilemap_post)(cmds)
    }
}

pub fn tile_builder_provider<T: DeserializeOwned>(
    tile_builder: impl FnMut(&T, &mut EntityCommands) -> anyhow::Result<()>,
    tileset_callback: impl FnMut(&T, &Tileset, &TilesetIndexing) -> anyhow::Result<()>,
    tilemap_post: impl FnMut(&mut EntityCommands) -> anyhow::Result<()>,
) -> impl TileBuilderProvider {
    TileBuilderProviderImpl {
        tilemap_post,
        tile_builder,
        tileset_callback,
        deserialized_props: HashMap::new(),
    }
}

pub fn parse_map<'a>(
    commands: &mut Commands,
    indexing: &[TilesetIndexing],
    atlases: &[Handle<Image>],
    map: &Map,
    callback_selector: impl Fn(&Tileset) -> &'a mut dyn TileBuilderProvider,
) -> anyhow::Result<()> {
    // Visit all tilesets
    for (id, set) in map.tilesets().iter().enumerate() {
        callback_selector(set).process_tileset(id, set, &indexing[id])?;
    }

    let mut map_commands = commands.spawn();
    map_commands
        .insert_bundle(TransformBundle::default())
        .insert_bundle(VisibilityBundle::default())
        .insert(Name::new("Map"));
    let mut result = Ok(());
    map_commands.with_children(|builder| {
        for layer in map.layers() {
            let mut layer_cmds = builder.spawn();
            layer_cmds
                .insert_bundle(TransformBundle::default())
                .insert_bundle(VisibilityBundle::default())
                .insert(Name::new(layer.name.clone()));
            let local_res = parse_layer(
                &mut layer_cmds,
                indexing,
                atlases,
                layer,
                &callback_selector
            );
            if local_res.is_err() {
                result = local_res.context(format!("While parsing layer {:?}", layer.name));
                return;
            }
        }
    });

    result
}

fn parse_layer<'a, F>(
    layer_cmds: &mut EntityCommands,
    indexing: &[TilesetIndexing],
    atlases: &[Handle<Image>],
    layer: Layer,
    callback_selector: &F,
) -> anyhow::Result<()> 
where
    F: Fn(&Tileset) -> &'a mut dyn TileBuilderProvider,
{
    // Start visitting layers
    match layer.layer_type() {
        LayerType::GroupLayer(group) => {
            let mut result = Ok(());

            // Spawn the children layers
            layer_cmds.with_children(|child_builder| {
                // Setup the parent components (transform and name)
                let mut layer_cmds = child_builder.spawn();
                layer_cmds    
                    .insert_bundle(TransformBundle::default())
                    .insert_bundle(VisibilityBundle::default())
                    .insert(Name::new(layer.name.clone()));

                for layer in group.layers() {
                    let local_res = parse_layer(
                        &mut layer_cmds, 
                        indexing,
                        atlases,
                        layer, 
                        callback_selector
                    );
                        
                    if local_res.is_err() {
                        result = local_res.context(format!("While parsing layer {:?}", layer.name));
                        return;
                    }
                }
            });

            result
        },
        LayerType::ImageLayer(_) => bail!("Image layers are not supported"),
        LayerType::ObjectLayer(_) => bail!("Objetc layers are not supported"),
        LayerType::TileLayer(tile_layer) => match tile_layer {
            TileLayer::Finite(tiles) => parse_finite_tile_layer(
                layer_cmds, 
                indexing, 
                atlases, 
                tiles, 
                callback_selector,
            ),
            TileLayer::Infinite(_) => bail!("Infinite tile layers are not supported")
        } 
    }
}

fn parse_finite_tile_layer<'a, F>(
    layer_cmds: &mut EntityCommands,
    indexing: &[TilesetIndexing],
    atlases: &[Handle<Image>],
    tiles: FiniteTileLayer,
    callback_selector: &F,
) -> anyhow::Result<()> 
where
    F: Fn(&Tileset) -> &'a mut dyn TileBuilderProvider,
{
    let (tileset_index, tileset) = ensure_unique_tileset(&tiles)?;
    let provider = callback_selector(tileset);
    let mut result = Ok(());
    let tilemap_size = TilemapSize { x: tiles.map().width, y: tiles.map().height };
    let mut storage = TileStorage::empty(tilemap_size);
    let parent_id = layer_cmds.id();

    // Spawn the tile
    layer_cmds.with_children(|builder| {
        for x in 0..tiles.map().width {
            for y in 0..tiles.map().height {
                let tile = match tiles.get_tile_data(x as i32, y as i32) {
                    Some(x) => x,
                    None => continue,
                };

                match spawn_tile(
                    x, 
                    tiles.map().height - 1 - y, 
                    parent_id, 
                    tileset_index, 
                    tile, 
                    indexing, 
                    builder, 
                    provider
                ) {
                    Ok((pos, id)) => storage.set(&pos, Some(id)),
                    Err(e) => {
                        result = Err(e).context(format!("Error while spawning tile ({}, {})", x, y));
                        return;
                    }
                }
            }
        }
    });

    layer_cmds
        .insert_bundle(TilemapBundle {
            storage,
            texture: TilemapTexture(atlases[tileset_index].clone()),
            mesh_type: TilemapMeshType::Square,
            tile_size: TilemapTileSize { 
                x: tileset.tile_width as f32, 
                y: tileset.tile_height as f32, 
            },
            grid_size: TilemapGridSize { 
                x: tileset.tile_width as f32, 
                y: tileset.tile_height as f32, 
            },
            size: tilemap_size,
            ..default()
        });  

    provider.tilemap_post_build(layer_cmds)?;

    result
}

fn ensure_unique_tileset<'a>(layer: &'a tiled::FiniteTileLayer) -> Result<(usize, &'a Tileset), anyhow::Error> {
    let mut result = None;

    for x in 0..layer.map().width {
        for y in 0..layer.map().height {
            if let Some(tile) = layer.get_tile(x as i32, y as i32) {
                let tileset = (tile.tileset_index(), tile.get_tileset());

                ensure!(
                    result.is_none() || result == Some(tileset), 
                    "The tileset is using more than one tileset"
                );
                result = Some(tileset);
            }
        }
    }

    result.ok_or_else(|| anyhow!("The layer uses no tileset"))
}

fn spawn_tile(
    x: u32,
    y: u32,
    parent_id: Entity,
    tileset_index: usize,
    tile: &LayerTileData,
    indexing: &[TilesetIndexing],
    builder: &mut ChildBuilder,
    provider: &mut dyn TileBuilderProvider,
) -> anyhow::Result<(TilePos, Entity)> {
    let position = TilePos { x, y };
    let mut tile_commands = builder.spawn_bundle(TileBundle {
        position,
        tilemap_id: TilemapId(parent_id),
        texture: TileTexture(indexing[tileset_index].dispatch(tile.id())),
        flip: tile.bevy_flip_flags(),
        ..default()
    });

    tile_commands.insert(Name::new("Tile"));
    provider.run_builder(tileset_index, tile.id(), &mut tile_commands)?;

    Ok((position, tile_commands.id()))
}