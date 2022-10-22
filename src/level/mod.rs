mod resources;

use std::{time::Duration, path::Path};
use std::collections::HashMap;

use bevy::asset::AssetPath;
use bevy_asset_loader::dynamic_asset::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::{prelude::*, asset::HandleId};

use bevy_ecs_tilemap_cpu_anim::{CPUTileAnimation, Frame};
use cube_rot::MoveDirection;
pub use resources::*;

use crate::tile::*;
use serde::Deserialize;
use bevy_tiled::{ TiledPlugin, TiledMap, TilesetIndexing, TileBuilder, TileExt, parse_map, SimpleCallbackSelector };
use crate::moveable::MoveableTilemapTag;

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(TiledPlugin)
            .add_plugin(TilemapPlugin)
            .add_plugin(TilePlugin);
    }
}

pub fn tile_pos_to_world_pos(
    tile_pos: TilePos,
    map_transform: &Transform,
    map_grid: &TilemapGridSize,
) -> Vec2 {
    map_transform.mul_vec3(Vec3::new(
        tile_pos.x as f32 * map_grid.x + map_grid.x / 2.0f32, 
        tile_pos.y as f32 * map_grid.y + map_grid.y / 2.0f32, 
        0.0f32
    )).truncate()
}

pub fn queue_level_tileset_images(
    base_level_assets: Res<BaseLevelAssets>,
    maps: Res<Assets<TiledMap>>,
    mut asset_keys: ResMut<DynamicAssets>,
) {
    info!("Queueing images");
    let map = maps.get(&base_level_assets.map).unwrap();
    asset_keys.register_asset("tileset_images", Box::new(map.get_tileset_dynamic_asset()));
}

pub fn get_level_map(
    base_level_assets: Res<BaseLevelAssets>,
    level_tilesets: Res<LevelTilesetImages>, 
) -> (Handle<TiledMap>, Vec<Handle<TextureAtlas>>) {
    (
        base_level_assets.map.clone(),
        level_tilesets.images.clone()
    )
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct TileAnimationFrame {
    id: u32,
    dur: u64,
}

#[derive(Debug, Deserialize)]
struct TileAnimation(Vec<TileAnimationFrame>);

impl TileAnimation {
    pub fn into_cpu_tile_anim(
        self, 
        mapping: &TilesetIndexing,
    ) -> CPUTileAnimation {
        CPUTileAnimation::new(
            self.0.into_iter()
                .map(|frame| Frame {
                    texture_id: mapping.dispatch(frame.id),
                    duration: Duration::from_millis(frame.dur),
                })
        )
    }
}

impl TileAnimation {
    fn decode(self, indexing: &TilesetIndexing) -> CPUTileAnimation {
        CPUTileAnimation::new(
            self.0.into_iter()
                .map(|frame| Frame {
                    texture_id: indexing.dispatch(frame.id),
                    duration: Duration::from_millis(frame.dur),
                })
        )
    }

    fn acquire_asset(
        self, 
        tileset: usize,
        tile: u32,
        assets: &mut Assets<CPUTileAnimation>, 
        map_path: Option<&Path>,
        indexing: &TilesetIndexing,
    ) -> Handle<CPUTileAnimation> {
        let asset = Self::decode(self, indexing);

        match map_path {
            Some(path) => assets.set(AssetPath::new_ref(
                path, Some(&format!("anim{}_{}", tileset, tile))
            ), asset),
            None => assets.add(asset),
        }
    }
}

impl ActivatableAnimating<TileAnimation> {
    fn decode(
        self,
        tileset: usize,
        tile: u32,
        assets: &mut Assets<CPUTileAnimation>, 
        map_path: Option<&Path>,
        indexing: &TilesetIndexing,
    ) -> ActivatableAnimating {
        self.convert(move |anim| anim.acquire_asset(tileset, tile, assets, map_path, indexing))
    }
}

struct GraphicsTileBuilder<'a> {
    map_path: Option<&'a Path>,
    anims: &'a mut Assets<CPUTileAnimation>,
    deserialized_props: HashMap<(usize, u32), GraphicsTileBundle>,
}

impl<'a> TileBuilder for GraphicsTileBuilder<'a> {
    fn process_tileset(
        &mut self, 
        set_id: usize, 
        tileset: &tiled::Tileset, 
        indexing: &TilesetIndexing,
    ) -> anyhow::Result<()> {
        self.deserialized_props.reserve(tileset.tilecount as usize);

        for (id, tile) in tileset.tiles() {
            let props: GraphicsTileBundle<TileAnimation> = tile.properties()?;
            self.deserialized_props.insert(
                (set_id, id), 
                GraphicsTileBundle { 
                    animating: props.animating.decode(
                        set_id, 
                        id, 
                        self.anims, 
                        self.map_path, 
                        indexing
                    ), 
                    anim: props.anim, 
                }
            );
        }

        Ok(())
    }

    fn run_builder(
        &mut self, 
        set_id: usize, 
        id: u32, 
        cmds: &mut bevy::ecs::system::EntityCommands,
    ) -> anyhow::Result<()> {
        cmds.insert_bundle(self.deserialized_props[&(set_id, id)].clone());

        Ok(())
    }

    fn tilemap_post_build(
        &mut self, 
        _set_id: usize, 
        cmds: &mut bevy::ecs::system::EntityCommands,
    ) -> anyhow::Result<()> {
        cmds
            .insert(GraphicsTilemapTag)
            .insert_bundle(TransformBundle::from_transform(Transform::from_scale(Vec3::new(
                1.6f32, 1.6f32, 1.6f32
            ))));

        Ok(())
    }
}

pub fn init_level_resource(
    In(tileset_indexing): In<Vec<TilesetIndexing>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands, 
    base_level_assets: Res<BaseLevelAssets>,
    tilesets: Res<LevelTilesetImages>,
    atlases: Res<Assets<TextureAtlas>>,
    maps: Res<Assets<TiledMap>>,
    mut animations: ResMut<Assets<CPUTileAnimation>>,
) {
    use bevy_tiled::BasicDeserBuilder;

    let mut logic_tile_builder = BasicDeserBuilder::<LogicTileBundle, _>::new(|cmds| { 
        cmds
            .insert(LogicTilemapTag)
            .insert(MoveableTilemapTag)
            .insert_bundle(TransformBundle::from_transform(Transform::from_scale(Vec3::new(
                1.6f32, 1.6f32, 1.6f32
            ))))
            .insert(Visibility { is_visible: false });
    });
    let mut trigger_tile_builder = BasicDeserBuilder::<TriggerTileBundle, _>::new(|cmds| { 
        cmds
            .insert(TriggerTilemapTag)
            .insert_bundle(TransformBundle::from_transform(Transform::from_scale(Vec3::new(
                1.6f32, 1.6f32, 1.6f32
            ))))
            .insert(Visibility { is_visible: false });
    });
    let map_asset_path = asset_server.get_handle_path(base_level_assets.map.clone());
    let mut graphics_tile_builder = GraphicsTileBuilder {
        map_path: map_asset_path.as_ref().map(|x| x.path()),
        anims: &mut *animations,
        deserialized_props: HashMap::new(),
    };
    let map = maps.get(&base_level_assets.map).unwrap();
    let atlases = tilesets.images.iter()
        .map(|x| atlases.get(&x).unwrap().texture.clone())
        .collect::<Vec<_>>(); 

    let res = parse_map(
        &mut commands, 
        &tileset_indexing, 
        &atlases, 
        &map.map, 
        &mut SimpleCallbackSelector {
            pool: [
                &mut logic_tile_builder, 
                &mut trigger_tile_builder, 
                &mut graphics_tile_builder
            ],
            picker: |name| match name {
                "logic" => 0,
                "triggers" => 1,
                _ => 2,
            },
        }
    );
    if let Err(e) = res {
        error!("Error parsing map: {}", e);
    }
}

pub fn spawn_level(
    mut commands: Commands, 
) {}
