mod resources;

use bevy_ecs_tilemap::prelude::*;
use bevy::prelude::*;

pub use resources::*;

use crate::{tile::*, moveable::MoveableTilemapTag};

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(TilemapPlugin)
            .add_plugin(TilePlugin);
    }
}

pub fn tile_pos_to_world_pos(
    tile_pos: TilePos,
    map_transform: &Transform,
    map_grid: &TilemapGridSize,
) -> Vec2 {
    map_transform.transform_point(
        tile_pos.center_in_world(
            map_grid,
            &TilemapType::Square,
        ).extend(0.0f32)
    ).truncate()
}

// NOTE I don't think I can do anything here to satisfy clippy.
// Maybe some further investigation will prove me wrong.
pub fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let texture_handle: Handle<Image> = asset_server.load("assets/tiles/atlas.png");
    let mut storage = TileStorage::empty(TilemapSize { x: 3, y: 3 });

    let tilemap_id = commands.spawn_empty().id();
    let tile = commands.spawn((
        TileBundle {
            tilemap_id: TilemapId(tilemap_id),
            ..default()
        },
        LogicKind::Start,
    ))
    .id();
    storage.set(
        &TilePos { x: 0, y: 0 },
        tile,
    );
    commands.entity(tilemap_id).insert((
        Name::new("Level"),
        TilemapBundle {
            storage,
            texture: TilemapTexture::Single(texture_handle),
            grid_size: TilemapTileSize { x: 32.0, y: 32.0 }.into(),
            tile_size: TilemapTileSize { x: 32.0, y: 32.0 },
            ..TilemapBundle::default()
        },
        MoveableTilemapTag,
    ))
    .add_child(tile);
}