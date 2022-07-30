mod assets;
mod components;
mod resources;
mod tile_decoder;

use std::collections::HashMap;

use bevy_asset_loader::*;
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
            .add_system(add_set_texture_filter_to_nearest)
            .add_plugin(TilePlugin);
    }
}

// TODO can we avoid doing that? What is the purpose of it
// anyway?
// Recommended by bevy_ecs_tilemap
fn add_set_texture_filter_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    use bevy::render::render_resource::TextureUsages;

    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.texture_descriptor.usage = 
                        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::COPY_DST;
                }
            }
            _ => (),
        }
    }
}

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
        DynamicAsset::Files {
            paths: level.get_used_images(),
        }
    );
}
    
fn spawn_layer_tiles(
    level: &Level,
    tileset_index: usize,
    layer: tiled::Layer,
    commands: &mut Commands,
    builder: &mut LayerBuilder<TileBundle>,
    tile_init: &tile_decoder::TileCommands,
) {
    for x in 0..level.map.width {
        for y in 0..level.map.height {
            match layer.layer_type() {
                tiled::LayerType::TileLayer(tile_layer) => {
                    tile_layer.get_tile(x as i32, y as i32).map(|tile| {
                        let y = (level.map.height - 1) as u32 - y;
                        let pos = TilePos(x, y);

                        // Skip tiles which don't use the tileset we 
                        // are considering on the current iteration
                        if tile.tileset_index() != tileset_index {
                            return;
                        }

                        let entity = builder.get_tile_entity(commands, pos).unwrap();
                        tile_init[tileset_index as usize][&tile.id()](commands.entity(entity));

                        // NOTE who told me that this is correct? Nobody. I am going to
                        // research if it's possible to break this code or not
                        let tile = 
                            Tile {
                                texture_index: level.get_tile_texture(tileset_index, tile.id()),
                                flip_x: tile.flip_h,
                                flip_y: tile.flip_v,
                                flip_d: tile.flip_d,
                                ..Default::default()
                            };

                        builder.set_tile(pos, TileBundle { tile, ..default() }).unwrap();
                    });
                },
                _ => panic!("Unsupported layer type"),
            }
        }
    }
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
    // The data we start with
    let map_id = 0;
    let level = levels.get(&base_level_assets.level).unwrap();

    // Parse the map
    let attrs = tile_decoder::scan_tilesets(&level.map);
    let anims = tile_decoder::scan_tilesets_for_animations(&level.map, &attrs);
    let mut anim_ids = vec![HashMap::new(); attrs.len()];
    let tile_init = tile_decoder::build_commands(&attrs, &anims, &mut anim_ids, &mut *animations);

    // Create the loaded map
    let map_entity = commands.spawn().insert(Name::new("Map")).id();
    let mut map = Map::new(map_id, map_entity);

    // Account for each tileset
    for (tileset_index, tileset) in level.map.tilesets().iter().enumerate() {
        let tileset_handle = 
            level.tilesets
                .get(&tileset_index)
                .expect("The tileset seems to be absent in the level data")
                .ready_image();
        let texture_size = 
            textures.get(&tileset_handle)
            .expect("The tileset image hasn't been loaded")
            .size()
            .to_array()
        ;
        // Loop through each layer
        for (layer_index, layer) in level.map.layers().enumerate() {
            let tile_width = tileset.tile_width as f32;
            let tile_height = tileset.tile_height as f32;

            let offset_x = layer.offset_x;
            let offset_y = layer.offset_y;

            let mut map_settings = LayerSettings::new(
                MapSize(
                    (level.map.width as f32 / 64.0).ceil() as u32,
                    (level.map.height as f32 / 64.0).ceil() as u32,
                ),
                ChunkSize(64, 64),
                TileSize(tile_width, tile_height),
                TextureSize(texture_size[0], texture_size[1]),
            );

            map_settings.grid_size = Vec2::new(
                level.map.tile_width as f32,
                level.map.tile_height as f32,
            );

            map_settings.mesh_type = match level.map.orientation {
                tiled::Orientation::Orthogonal => TilemapMeshType::Square,
                _ => panic!("Bad tile format"),
            };

            let (mut builder, layer_entity) = LayerBuilder::<TileBundle>::new(
                &mut commands,
                map_settings.clone(),
                map_id,
                layer_index as u16
            );

            spawn_layer_tiles(&*level, tileset_index, layer, &mut commands, &mut builder, &tile_init);

            let layer_bundle = 
                builder.build(
                    &mut commands,
                    &mut meshes,
                    tileset_handle.clone()
                );

            commands.entity(layer_entity)
            .insert_bundle(layer_bundle).insert(Transform::from_xyz(
                offset_x,
                -offset_y,
                layer_index as f32,
            ));
            map.add_layer(&mut commands, layer_index as u16, layer_entity);
        }
    }

    commands.entity(map_entity)
        .insert(map)
        // TODO make the crash into a `warn!(...)`
        .insert(LevelInfo {
            map: map_id,
            geometry_layer: level.find_geometry_layer().expect("No geometry layer"),
        })
        .insert_bundle(TransformBundle::from_transform(
            Transform::from_scale(Vec3::new(1.6f32, 1.6f32, 1.6f32))
        ));
}
