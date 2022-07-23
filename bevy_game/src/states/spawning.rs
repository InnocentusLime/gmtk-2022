use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use super::GameState;
use super::ingame::{ InGameAssets, PlayerGraphicsResources };

use crate::level::*;
use crate::player::*;

fn init_level(
    mut commands: Commands,
    animations: ResMut<Animations>,
    meshes: ResMut<Assets<Mesh>>,
    mut ready: EventWriter<MapReady>,
    levels: Res<Assets<Level>>,
    assets: Res<InGameAssets>,
) {
    info!("Initting level");

    let level = levels.get(&assets.level).unwrap();
    match level.find_geometry_layer_id() {
        Some(layer_id) => {
            let (map_id, map_entity) = level.spawn_map(&mut commands, meshes, animations);
            let map_tf = Transform::from_scale(Vec3::new(1.6f32, 1.6f32, 1.0f32));
            commands.entity(map_entity).insert_bundle(TransformBundle::from_transform(map_tf.clone()));

            ready.send(MapReady { map_id, layer_id });
        },
        None => error!("Failed to find a layer called \"geometry\". Make sure your level file has a layer named \"geometry\""),
    }
}

fn init_objects(
    mut map_ready_event: EventReader<MapReady>,
    mut commands: Commands,
    start_q: Query<&TilePos, With<StartTileTag>>,
    player_gfx: Res<PlayerGraphicsResources>,
    mut map: MapQuery,
    map_entity: Query<&Transform, With<Map>>,
) {
    use bevy::sprite::MaterialMesh2dBundle;

    info!("Initing objects");
    for ready in map_ready_event.iter() {
        let map_tf = map_entity.single();
        for pos in start_q.iter() {
            let world_pos = tile_pos_to_world_pos(
                (pos.0, pos.1), 
                map_tf, 
                &mut map, 
                ready.map_id, 
                ready.layer_id
            );
            commands.spawn()
                .insert(Name::new("Player"))
                .insert(PlayerState::AwaitingInput)
                .insert(Player::new((pos.0, pos.1), ready.map_id, ready.layer_id))
                .insert_bundle(MaterialMesh2dBundle {
                    mesh: player_gfx.model.clone(),
                    material: player_gfx.material.clone(),
                    transform: Transform::from_translation(world_pos.extend(1.0f32))
                        .with_scale(Vec3::new(25.0f32, 25.0f32, 25.0f32)),
                    ..default()
                });
        }
    }

    commands.insert_resource(NextState(GameState::InGame));
}

pub fn setup_states(app: &mut App) {
    app
        .add_enter_system(GameState::Spawning, init_level)
        .add_system(init_objects.run_in_state(GameState::Spawning));
}
