mod resources;

use std::error::Error;
use std::collections::HashMap;

use bevy_asset_loader::{ standard_dynamic_asset::*, dynamic_asset::* };
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimations;
use bevy::prelude::*;

pub use resources::*;

use crate::tile::*;
use bevy_tiled::{ TiledPlugin, TiledMap, TilesetsFromTiled, TilesetIndexing, TiledTileset };
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

pub fn init_level_resource(
    In(tileset_indexing): In<Vec<TilesetIndexing>>,
    mut commands: Commands, 
    base_level_assets: Res<BaseLevelAssets>,
    tilesets: Res<LevelTilesetImages>,
    atlases: Res<Assets<TextureAtlas>>,
    maps: Res<Assets<TiledMap>>,
) {
    let map = maps.get(&base_level_assets.map).unwrap();
    match Level::new(tileset_indexing, map, &*tilesets, &*atlases) {
        Ok(level) => { commands.insert_resource(level); },
        Err(e) => e.chain().for_each(|e| error!("{}", e)),
    }
}

// FIXME ugly nested stuff
pub fn spawn_level(
    mut commands: Commands, 
    textures: Res<Assets<Image>>,
    level: Res<Level>,
    //mut animations: ResMut<CPUTileAnimations>,
) {
    // Create the loaded map
    let map_entity = commands.spawn().insert(Name::new("Map")).id();
    let tilemap_size = TilemapSize { x: level.width(), y: level.height() };
    let mut tile_store = TileStorage::empty(tilemap_size);

    // Build the geometry layer
    let map_commands = 
        commands.entity(map_entity)
            .with_children(|commands| {
                for x in 0..level.width() {
                    for y in 0..level.height() {
                        // Skip a tile if it has no graphics.
                        if level.geometry_graphics.get(&(x, y)).is_none() { continue; }

                        let tile_pos = TilePos { x, y };
                        let mut cmds = commands.spawn();

                        cmds.insert_bundle(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(map_entity),
                            texture: TileTexture(level.geometry_graphics[&(x, y)]),
                            flip: level.level_tiles_flip.get(&(x, y)).map(|f| *f).unwrap_or_default(),
                            ..default()
                        }).insert(Name::new("level tile"));

                        if let Some(ty) = level.level_tiles.get(&(x, y)) {
                            ty.insert_into(&mut cmds);
                        }

                        if let Some(act_cond) = level.activators.get(&(x, y)) {
                            cmds
                                .insert(*act_cond)
                                .insert(Active { is_active: true });
                        }

                        tile_store.set(&tile_pos, Some(cmds.id()));
                    }
                }
            });

    commands.entity(map_entity)
        .insert_bundle(TilemapBundle {
            storage: tile_store,
            texture: TilemapTexture(level.geometry_atlas.clone()),
            mesh_type: TilemapMeshType::Square,
            // FIXME hardcoded
            tile_size: TilemapTileSize { x: 32.0f32, y: 32.0f32 },
            // FIXME hardcoded
            grid_size: TilemapGridSize { x: 32.0f32, y: 32.0f32 },
            size: tilemap_size,
            transform: Transform::from_scale(Vec3::new(1.6f32, 1.6f32, 1.6f32)),
            ..default()
        })
        .insert(MoveableTilemapTag);
}
