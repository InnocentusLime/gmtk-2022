use anyhow::{anyhow, bail, ensure, Context};
use bevy_ecs_tilemap::{tiles::{TileBundle, TilePos, TileTextureIndex, TileStorage}, prelude::{TilemapId, TilemapSize, TilemapTexture, TilemapType, TilemapTileSize, TilemapGridSize}, TilemapBundle};
use std::collections::HashMap;

use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::de::DeserializeOwned;
use tiled::{Tileset, Map, Layer, LayerType, TileLayer, FiniteTileLayer, LayerTileData};

use crate::{TileExt, TilesetIndexing, TiledLayerTileExt};

/// An interface for the tilemap parser to call as it visits different
/// parts of the tilemap asset.
pub trait TileBuilder {
    /// Gets called by the parser for each tilesets **before** processing
    /// the layers and tiles.
    fn process_tileset(
        &mut self,
        set_id: usize,
        tileset: &Tileset,
        indexing: &TilesetIndexing
    ) -> anyhow::Result<()>;

    /// Gets called for each tile on each layer.
    fn build(
        &mut self,
        set_id: usize,
        id: u32,
        cmds: &mut EntityCommands
    ) -> anyhow::Result<()>;

    /// Gets called by the parser after finishing a layer.
    fn finish_layer(
        &mut self,
        set_id: usize,
        cmds: &mut EntityCommands
    ) -> anyhow::Result<()>;
}

pub struct BasicDeserBuilder<T, F = fn(&mut EntityCommands)>
where
    T: DeserializeOwned + Bundle,
    F: Fn(&mut EntityCommands),
{
    bundle_builder: F,
    deserialized_props: HashMap<(usize, u32), T>,
}

impl<T, F> BasicDeserBuilder<T, F>
where
    T: DeserializeOwned + Bundle,
    F: Fn(&mut EntityCommands),
{
    pub fn new(bundle_builder: F) -> Self {
        BasicDeserBuilder {
            bundle_builder,
            deserialized_props: HashMap::new(),
        }
    }
}

impl<T> BasicDeserBuilder<T, fn(&mut EntityCommands)>
where
    T: DeserializeOwned + Bundle,
{
    pub fn new_default<B: Bundle + Default>() -> Self {
        BasicDeserBuilder::new(|cmds| { cmds.insert(B::default()); })
    }
}

impl<T, F> TileBuilder for BasicDeserBuilder<T, F>
where
    T: DeserializeOwned + Bundle + Clone,
    F: Fn(&mut EntityCommands),
{
    fn process_tileset(
        &mut self,
        set_id: usize,
        tileset: &Tileset,
        _indexing: &TilesetIndexing
    ) -> anyhow::Result<()> {
        self.deserialized_props.reserve(tileset.tilecount as usize);

        for (id, tile) in tileset.tiles() {
            let props = tile.properties()?;
            self.deserialized_props.insert((set_id, id), props);
        }

        Ok(())
    }

    fn build(
        &mut self,
        set_id: usize,
        id: u32,
        cmds: &mut EntityCommands
    ) -> anyhow::Result<()> {
        let props = self.deserialized_props.get(&(set_id, id))
            .ok_or_else(|| anyhow!("Tile {} didn't have any deserialized properties", id))?;

        cmds.insert(props.clone());

        Ok(())
    }

    fn finish_layer(
        &mut self,
        _set_id: usize,
        cmds: &mut EntityCommands
    ) -> anyhow::Result<()> {
        (self.bundle_builder)(cmds);

        Ok(())
    }
}

pub trait CallbackSelector {
    fn select(&mut self, tileset: &Tileset) -> &mut dyn TileBuilder;
}

pub struct SimpleCallbackSelector<'a, const N: usize> {
    pub pool: [&'a mut dyn TileBuilder; N],
    pub picker: fn(&str) -> usize,
}

impl<'a, const N: usize> CallbackSelector for SimpleCallbackSelector<'a, N> {
    fn select(&mut self, tileset: &Tileset) -> &mut dyn TileBuilder {
        self.pool[(self.picker)(&tileset.name)]
    }
}

struct ParserState {
    layer_idx: u32,
}

impl ParserState {
    fn new() -> Self {
        Self {
            layer_idx: 0,
        }
    }
}

pub struct MapParser<'w, 's, 'a, C> {
    state: ParserState,
    commands: &'a mut Commands<'w, 's>,
    callback_selector: C,
    tilemap_texture_data: &'a [(TilesetIndexing, TilemapTexture)],
}

impl<'w, 's, 'a, C: CallbackSelector> MapParser<'w, 's, 'a, C>
where
    'w: 'a,
    's: 'a,
{
    pub fn new(
        commands: &'a mut Commands<'w, 's>,
        callback_selector: C,
        tilemap_texture_data: &'a [(TilesetIndexing, TilemapTexture)],
    ) -> Self {
        Self {
            state: ParserState::new(),
            commands,
            callback_selector,
            tilemap_texture_data,
        }
    }

    pub fn parse_map(&mut self, map: &Map) -> anyhow::Result<()> {
        for (id, set) in map.tilesets().iter().enumerate() {
            self.callback_selector.select(set).process_tileset(
                id,
                set,
                &self.tilemap_texture_data[id].0
            )?;
        }

        let callback_selector = &mut self.callback_selector;
        let tilemap_texture_data = &self.tilemap_texture_data;

        let mut result = Ok(());
        self.commands.spawn((
            TransformBundle::default(),
            VisibilityBundle::default(),
            Name::new("Map"),
        ))
        .with_children(|builder| {
            for layer in map.layers() {
                let mut layer_cmds = builder.spawn((
                    TransformBundle::default(),
                    VisibilityBundle::default(),
                    Name::new(layer.name.clone()),
                ));
                let local_res = Self::parse_layer(
                    &mut self.state,
                    &mut layer_cmds,
                    callback_selector,
                    tilemap_texture_data,
                    layer,
                );
                if local_res.is_err() {
                    result = local_res.context(format!("While parsing layer {:?}", layer.name));
                    return;
                }
            }
        });

        result
    }

    fn parse_layer(
        state: &mut ParserState,
        layer_cmds: &mut EntityCommands,
        callback_selector: &mut C,
        tilemap_texture_data: &[(TilesetIndexing, TilemapTexture)],
        layer: Layer
    ) -> anyhow::Result<()> {
        // Start visitting layers
        match layer.layer_type() {
            LayerType::GroupLayer(group) => {
                let mut result = Ok(());

                // Spawn the children layers
                layer_cmds.with_children(|child_builder| {

                    let local_res = group.layers()
                    .try_for_each(|layer| {
                        let mut layer_cmds = child_builder.spawn((
                            TransformBundle::default(),
                            VisibilityBundle::default(),
                            Name::new(layer.name.clone()),
                        ));

                        Self::parse_layer(
                            state,
                            &mut layer_cmds,
                            callback_selector,
                            tilemap_texture_data,
                            layer,
                        )
                    });

                    if local_res.is_err() {
                        result = local_res;
                    }
                });

                result
            },
            LayerType::ImageLayer(_) => bail!("Image layers are not supported"),
            LayerType::ObjectLayer(_) => bail!("Objetc layers are not supported"),
            LayerType::TileLayer(tile_layer) => match tile_layer {
                TileLayer::Finite(tiles) => Self::parse_finite_tile_layer(
                    state,
                    layer_cmds,
                    callback_selector,
                    tilemap_texture_data,
                    tiles,
                ),
                TileLayer::Infinite(_) => bail!("Infinite tile layers are not supported")
            }
        }
    }

    fn parse_finite_tile_layer(
        state: &mut ParserState,
        layer_cmds: &mut EntityCommands,
        callback_selector: &mut C,
        tilemap_texture_data: &[(TilesetIndexing, TilemapTexture)],
        tiles: FiniteTileLayer,
    ) -> anyhow::Result<()> {
        use itertools::iproduct;

        let (tileset_index, tileset) = ensure_unique_tileset(&tiles)?;
        let provider = callback_selector.select(tileset);
        let tilemap_size = TilemapSize { x: tiles.map().width, y: tiles.map().height };
        let mut storage = TileStorage::empty(tilemap_size);
        let parent_id = layer_cmds.id();

        let mut result = Ok(());
        // Spawn the tiles
        layer_cmds.with_children(|builder| {
            let local_res = iproduct!(0..tiles.map().width, 0..tiles.map().height)
            .filter_map(|(x, y)|
                tiles.get_tile_data(x as i32, y as i32)
                .map(|data| (x, y, data))
            )
            .try_for_each(|(x, y, tile)| {
                let (pos, e) = Self::spawn_tile(
                    state,
                    (x, tiles.map().height - 1 - y),
                    parent_id,
                    tileset_index,
                    tilemap_texture_data,
                    tile,
                    builder,
                    provider
                )
                .context(format!("Error while spawning tile ({x}, {y})"))?;

                storage.set(&pos, e);

                Ok(())
            });

            if local_res.is_err() {
                result = local_res;
            }
        })
        .insert(TilemapBundle {
            storage,
            texture: tilemap_texture_data[tileset_index].1.clone(),
            map_type: TilemapType::Square,
            tile_size: TilemapTileSize { x: tileset.tile_width as f32, y: tileset.tile_height as f32 },
            grid_size: TilemapGridSize { x: tileset.tile_width as f32, y: tileset.tile_height as f32 },
            size: tilemap_size,
            transform: Transform::from_translation(Vec3::new(
                0.0f32,
                0.0f32,
                state.layer_idx as f32,
            )),
            ..default()
        });

        provider.finish_layer(tileset_index, layer_cmds)?;
        ensure!(state.layer_idx < 100);
        state.layer_idx += 1;

        result
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_tile(
        _state: &mut ParserState,
        (x, y): (u32, u32),
        parent_id: Entity,
        tileset_index: usize,
        tilemap_texture_data: &[(TilesetIndexing, TilemapTexture)],
        tile: &LayerTileData,
        builder: &mut ChildBuilder,
        tile_builder: &mut dyn TileBuilder,
    ) -> anyhow::Result<(TilePos, Entity)> {
        let position = TilePos { x, y };
        let mut tile_commands = builder.spawn((
            TileBundle {
                position,
                tilemap_id: TilemapId(parent_id),
                texture_index: TileTextureIndex(
                    tilemap_texture_data[tileset_index].0
                    .dispatch(tile.id())
                ),
                flip: tile.bevy_flip_flags(),
                ..default()
            },
            Name::new("Tile"),
        ));

        tile_builder.build(tileset_index, tile.id(), &mut tile_commands)?;

        Ok((position, tile_commands.id()))
    }
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