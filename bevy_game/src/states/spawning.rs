use bevy::prelude::*;
use bevy_asset_loader::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use super::GameState;
use super::ingame::InGameAssets;

use crate::level::*;
use crate::special_tiles::*;
use crate::player::*;

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

    let map_tf = Transform::from_scale(Vec3::new(1.6f32, 1.6f32, 1.0f32));
    commands.entity(map_entity).insert_bundle(TransformBundle::from_transform(map_tf.clone()));

    commands.insert_resource(NextState(GameState::InGame));
}

fn init_objects(
    mut commands: Commands,
    start_q: Query<&TilePos, With<StartTileTag>>,
    assets: Res<InGameAssets>,
    //mut map: MapQuery,
    //map_entity: Query<&Transform, With<Map>>,
    // TODO remove
    gltfs: Res<Assets<bevy::gltf::Gltf>>,
    gltf_meshes: Res<Assets<bevy::gltf::GltfMesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
) {
    use bevy::render::mesh::shape::Cube;
    use bevy::sprite::Mesh2dHandle;
    use bevy::sprite::MaterialMesh2dBundle;
    info!("Initing objects");
    let player_gltf = gltfs.get(&assets.player_gltf).unwrap();
    let player_gltf = gltf_meshes.get(&player_gltf.meshes[0]).unwrap();
    let player_gltf = &player_gltf.primitives[0];
    //let map_tf = map_entity.single();
    for pos in start_q.iter() {
        //let world_pos = tile_pos_to_world_pos((pos.0, pos.1), map_tf, &mut map, 0, 0);
        let world_pos = Vec2::new(0.0f32, 0.0f32);
        commands.spawn()
            .insert(PlayerState::AwaitingInput)
            .insert(Player::new((pos.0, pos.1)))
            .insert_bundle(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(player_gltf.mesh.clone()),
                material: materials.add(ColorMaterial {
                    color: Color::WHITE,
                    texture: std_materials.get(&player_gltf.material.as_ref().unwrap().to_owned()).unwrap().base_color_texture.to_owned(),
                }),
                transform: Transform::from_translation(world_pos.extend(1.0f32))
                    .with_scale(Vec3::new(25.0f32, 25.0f32, 25.0f32)),
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
