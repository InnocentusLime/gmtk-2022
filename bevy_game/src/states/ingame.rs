use bevy::prelude::*;
use bevy_asset_loader::*;
use iyes_loopless::prelude::*;

use super::GameState;

use crate::level::*;
use crate::app::GameplayCamera;

#[derive(AssetCollection)]
pub struct InGameAssets {
    #[asset(path = "maps/test_map.tmx")]
    map: Handle<TiledMap>,
    #[asset(path = "tiles/level_tiles.png")]
    _atlas: Handle<Image>,
}

fn enter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    maps: Res<Assets<TiledMap>>,
    assets: Res<InGameAssets>,
    mut gameplay_camera: Query<&mut Transform, With<GameplayCamera>>,
) {
    info!("Entered ingame state");

    // Get the loaded map
    let tiled_map = maps.get(&assets.map).unwrap();
    let level_info = LevelInfo::new(&mut commands, meshes, tiled_map);
       
    let map_tf = Transform::from_scale(Vec3::new(3.0f32, 3.0f32, 1.0f32));
    commands.entity(level_info.map_entitiy())
        .insert_bundle(TransformBundle::from_transform(map_tf.clone()));
    info!("Player start {:?}", level_info.start_tile);

    let pos = tile_pos_to_world_pos(level_info.start_tile, tiled_map, &map_tf);
    info!("Player world position {:?}", pos);
    gameplay_camera.single_mut().translation = pos;
}

fn exit() {
    info!("Exited ingame state");
}

pub fn setup_states(app: &mut App) {
    app
        .add_enter_system(GameState::InGame, enter)
        .add_exit_system(GameState::InGame, exit);
}
