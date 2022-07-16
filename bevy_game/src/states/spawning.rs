use bevy::prelude::*;
use bevy_asset_loader::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use super::GameState;
use super::ingame::InGameAssets;

use crate::level::*;
use crate::player::PlayerState;

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    levels: Res<Assets<Level>>,
    assets: Res<InGameAssets>,
) {
    info!("Initting level");

    // Get the loaded map
    let level = levels.get(&assets.level).unwrap();
    let map_entity = level.spawn_map(&mut commands, meshes);

    let map_tf = Transform::from_scale(Vec3::new(3.0f32, 3.0f32, 1.0f32));
    commands.entity(map_entity).insert_bundle(TransformBundle::from_transform(map_tf.clone()));

    commands.insert_resource(NextState(GameState::InGame));
}

fn init_objects(
    mut commands: Commands,
    start_q: Query<&TilePos, With<StartTileTag>>,
    assets: Res<InGameAssets>,
    mut map: MapQuery,
    map_entity: Query<&Transform, With<Map>>,
) {
    info!("Initing objects");
    let map_tf = map_entity.single();
    for pos in start_q.iter() {
        let world_pos = tile_pos_to_world_pos((pos.0, pos.1), map_tf, &mut map, 0, 0);
        commands.spawn()
            .insert(PlayerState::new((pos.0, pos.1)))
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(45.0f32, 45.0f32)),
                    ..default()
                },
                texture: assets.player_texture.clone(),
                transform: Transform::from_translation(world_pos.extend(1.0f32)),
                ..default()
            });
    }
}

pub fn setup_states(app: &mut App) {
    use iyes_loopless::state::app::StateTransitionStageLabel;
    app
        .add_enter_system(GameState::Spawning, init_level)
        .add_exit_system(GameState::Spawning, init_objects);
}
