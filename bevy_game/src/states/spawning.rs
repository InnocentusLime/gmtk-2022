use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use super::GameState;
use super::ingame::{ InGameAssets, PlayerGraphicsResources };

use crate::level::*;
use crate::player::*;

fn init_level(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    ready: EventWriter<MapReady>,
    levels: Res<Assets<Level>>,
    assets: Res<InGameAssets>,
) {
    info!("Initting level");

    // Get the loaded map
    let level = levels.get(&assets.level).unwrap();
    let map_entity = level.spawn_map(ready, &mut commands, meshes);

    let map_tf = Transform::from_scale(Vec3::new(1.6f32, 1.6f32, 1.0f32));
    commands.entity(map_entity).insert_bundle(TransformBundle::from_transform(map_tf.clone()));
}

fn init_objects(
    mut commands: Commands,
    start_q: Query<&TilePos, With<StartTileTag>>,
    player_gfx: Res<PlayerGraphicsResources>,
    mut map: MapQuery,
    map_entity: Query<&Transform, With<Map>>,
) {
    use bevy::sprite::MaterialMesh2dBundle;

    info!("Initing objects");
    let map_tf = map_entity.single();
    for pos in start_q.iter() {
        let world_pos = tile_pos_to_world_pos((pos.0, pos.1), map_tf, &mut map, 0, 0);
        commands.spawn()
            .insert(PlayerState::AwaitingInput)
            .insert(Player::new((pos.0, pos.1)))
            .insert_bundle(MaterialMesh2dBundle {
                mesh: player_gfx.model.clone(),
                material: player_gfx.material.clone(),
                transform: Transform::from_translation(world_pos.extend(1.0f32))
                    .with_scale(Vec3::new(25.0f32, 25.0f32, 25.0f32)),
                ..default()
            });
    }

    commands.insert_resource(NextState(GameState::InGame));
}

pub fn setup_states(app: &mut App) {
    app
        .add_enter_system(GameState::Spawning, init_level)
        .add_system(init_objects.run_on_event::<MapReady>().run_in_state(GameState::Spawning));
}
