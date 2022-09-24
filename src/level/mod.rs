mod resources;

use bevy_asset_loader::dynamic_asset::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimations;
use bevy::prelude::*;

pub use resources::*;

use crate::tile::*;
use bevy_tiled::{ TiledPlugin, TiledMap, TilesetIndexing };
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
    mut animations: ResMut<CPUTileAnimations>,
) {
    let map = maps.get(&base_level_assets.map).expect("Level map should be loaded by now");
    let level = match Level::new(
        map, 
        &mut *animations,
        &tileset_indexing, 
        &tilesets.images.iter().filter_map(|h| atlases.get(h)).collect::<Vec<_>>(),
    ) {
        Ok(x) => x,
        Err(e) => { e.chain().for_each(|e| error!("{}", e)); return },
    };

    commands.insert_resource(level);
}

pub fn spawn_level(
    mut commands: Commands, 
    level: Res<Level>,
    animations: Res<CPUTileAnimations>,
) {
    // Create the loaded map
    let map_entity = commands.spawn().insert(Name::new("Map")).id();
    let tilemap_size = TilemapSize { x: level.width(), y: level.height() };
    let mut tile_store = TileStorage::empty(tilemap_size);

    // Build the geometry layer
    let _map_commands = 
        commands.entity(map_entity)
            .with_children(|commands| {
                level.tiles.iter().for_each(|((x, y), data)| {
                    let position = TilePos { x: *x, y: *y };
                    let mut cmds = commands.spawn();

                    cmds
                        .insert_bundle(TileBundle {
                            position,
                            tilemap_id: TilemapId(map_entity),
                            texture: data.texture,
                            flip: data.flip,
                            ..default()
                        })
                        .insert(Name::new("level tile"))
                        .insert(data.ty);

                    if let TileKind::Start = data.ty {
                        cmds.insert(StartTileTag);
                    }
                    
                    let enabled = data.activation_cond.map(ActivationCondition::active_on_start).unwrap_or(false);

                    if let Some(cond) = data.activation_cond {
                        cmds
                            .insert(cond)
                            .insert(TileState::Ready(cond.active_on_start()));
                    }

                    if let Some(animating) = data.activatable_animating {
                        match animating {
                            ActivatableAnimating::Switch { on_anim, off_anim, .. } => cmds.insert(
                                if enabled {
                                    animations.new_cpu_animated(on_anim, true, false)
                                } else {
                                    animations.new_cpu_animated(off_anim, true, false)
                                }
                            ),
                            ActivatableAnimating::Pause { anim } => cmds.insert(
                                animations.new_cpu_animated(anim, true, enabled)
                            ),
                        }.insert(animating);
                    }
                        
                    tile_store.set(&position, Some(cmds.id()));
                });
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
