mod resources;

use std::{time::Duration, path::Path};
use std::collections::HashMap;

use bevy::asset::AssetPath;
use bevy_asset_loader::dynamic_asset::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::{prelude::*};

use bevy_ecs_tilemap_cpu_anim::{CPUTileAnimation, Frame};
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

#[derive(Default, Clone, Copy, Debug, Deserialize)]
struct TileAnimationFrame {
    id: u32,
    dur: u64,
}

#[derive(Default, Debug, Deserialize)]
struct TileAnimation(Vec<TileAnimationFrame>);

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
        let mut acquire_asset = |anim, tag: &'static str| {
            let asset = TileAnimation::decode(anim, indexing);
            debug!("anim{tileset:}_{tile:}_{tag:}\n{asset:?}");

            match map_path {
                Some(path) => assets.set(AssetPath::new_ref(
                    path, Some(&format!("anim{tileset:}_{tile:}_{tag:}"))
                ), asset),
                None => assets.add(asset),
            }
        };

        match self {
            ActivatableAnimating::None => ActivatableAnimating::None,
            ActivatableAnimating::Pause { on_anim } => ActivatableAnimating::Pause { 
                on_anim: acquire_asset(on_anim, "on_anim"),
            },
            ActivatableAnimating::Switch { 
                on_transition, 
                off_transition, 
                on_anim, 
                off_anim,
            } => ActivatableAnimating::Switch { 
                on_transition: acquire_asset(on_transition, "on_transition"), 
                off_transition: acquire_asset(off_transition, "off_transition"), 
                on_anim: acquire_asset(on_anim, "off_anim"), 
                off_anim: acquire_asset(off_anim, "on_anim"), 
            }
        }
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

// NOTE I don't think I can do anything here to satisfy clippy.
// Maybe some further investigation will prove me wrong.
#[allow(clippy::too_many_arguments)]
pub fn spawn_level(
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

    let map_asset_path = asset_server.get_handle_path(base_level_assets.map.clone());

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
    let mut graphics_tile_builder = GraphicsTileBuilder {
        map_path: map_asset_path.as_ref().map(|x| x.path()),
        anims: &mut *animations,
        deserialized_props: HashMap::new(),
    };

    let map = maps.get(&base_level_assets.map).unwrap();
    let atlases = tilesets.images.iter()
        .map(|x| atlases.get(x).unwrap().texture.clone())
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
                "logic_tiles" => 0,
                "activator_tiles" => 1,
                _ => 2,
            },
        }
    );
    if let Err(e) = res {
        error!("Error parsing map: {}", e);
    }
}