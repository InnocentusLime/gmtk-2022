mod assets;
mod components;
mod resources;
mod tile_decoder;

use std::collections::HashMap;

use bevy_asset_loader::{ standard_dynamic_asset::*, dynamic_asset::* };
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimations;
use bevy::prelude::*;

pub use resources::*;
pub use components::*;
pub use assets::*;

use crate::tile::TilePlugin;

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Level>()
            .add_asset_loader(LevelLoader)
            .add_plugin(TilemapPlugin)
            .add_plugin(TilePlugin);
    }
}

/*
pub fn tile_pos_to_world_pos(
    tile_pos: (u32, u32),
    map_transform: &Transform,
    map: &mut MapQuery,
    map_id: u16,
    layer_id: u16,
) -> Vec2 {
    let layer = map.get_layer(map_id, layer_id).unwrap().1;
    let settings = &layer.settings;
    map_transform.mul_vec3(Vec3::new(
        tile_pos.0 as f32 * settings.tile_size.0 + settings.tile_size.0 / 2.0f32, 
        tile_pos.1 as f32 * settings.tile_size.1 + settings.tile_size.1 / 2.0f32, 
        0.0f32
    )).truncate()
}
*/

pub fn prepare_level_tileset_images(
    mut textures: ResMut<Assets<Image>>,
    base_level_assets: Res<BaseLevelAssets>,
    mut levels: ResMut<Assets<Level>>,
) {
    let level = levels.get_mut(&base_level_assets.level).unwrap();
    level.prepare_tilesets(&mut *textures);
}

pub fn queue_level_tileset_images(
    base_level_assets: Res<BaseLevelAssets>,
    levels: Res<Assets<Level>>,
    mut asset_keys: ResMut<DynamicAssets>,
) {
    let level = levels.get(&base_level_assets.level).unwrap();
    asset_keys.register_asset(
        "tileset_images",
        Box::new(StandardDynamicAsset::Files {
            paths: level.get_used_images(),
        })
    );
}

// TODO consider shrinking the nested stuff
pub fn spawn_level(
    mut commands: Commands, 
    textures: Res<Assets<Image>>,
    base_level_assets: Res<BaseLevelAssets>,
    levels: Res<Assets<Level>>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut animations: ResMut<CPUTileAnimations>,
) {
    todo!()

    /*
    // The data we start with
    let level = levels.get(&base_level_assets.level).unwrap();

    // Parse the map
    let attrs = tile_decoder::scan_tilesets(&level.map);
    let anims = tile_decoder::scan_tilesets_for_animations(&level.map, &attrs);
    let mut anim_ids = vec![HashMap::new(); attrs.len()];
    let tile_init = tile_decoder::build_commands(&attrs, &anims, &mut anim_ids, &mut *animations);
    let geometry_layer = level.find_geometry_layer().expect("No geometry layer");
    let geometry_layer = match level.map.tilesets()[geometry_layer].layer_type() {
        tiled::LayerType::TileLayer(x) => x,
        _ => panic!("Bad geometry layer type"),
    };

    // Create the loaded map
    let map_entity = commands.spawn().insert(Name::new("Map")).id();
    let tilemap_size = TilemapSize { x: level.map.width, y: level.map.height };
    let mut tile_store = TileStorage::empty(tilemap_size);

    // Build the geometry layer
    let mut tileset_id = None;
    for x in 0..level.map.width {
        for y in 0..level.map.height {
            tile_layer.get_tile(x as i32, y as i32).map(|tile| {
                let y = (level.map.height - 1) as u32 - y;
                let tile_pos = TilePos { x, y };

                let entity = commands.spawn()
                    .insert_bundle(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(map_id),
                        flip: TileFlip {
                            x: tile.flip_h,
                            y: tile.flip_v,
                            d: tile.flip_d,
                        },
                        ..default()
                    })
                    .id();

                tile_commands[tileset_index as usize][&tile.id()](commands.entity(entity));
                tile_storage.set(&tile_pos, Some(entity));
            });
        }
    }

    commands.entity(map_entity)
        .insert_bundle(TilemapBundle {
            storage: tile_store,
            texture: TilemapTexture(...),
            mesh_type: TilemapMeshType::Square,
            grid_size: TilemapGridSize { x:, y: }
            size: tilemap_size,
            transform: Transform::from_scale(Vec3::new(1.6f32, 1.6f32, 1.6f32)),
            ..default()
        });
    */
}
