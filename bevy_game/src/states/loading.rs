use bevy::prelude::*;
use bevy_asset_loader::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use super::GameState;
use crate::level::*;
use crate::player::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingLevelSubstate {
    LoadingBaseAssets,
    SpawningObjects,
}

fn spawn_level(
    mut commands: Commands,
    animations: ResMut<Animations>,
    meshes: ResMut<Assets<Mesh>>,
    mut ready: EventWriter<MapReady>,
    levels: Res<Assets<Level>>,
    base_level_assets: Res<BaseLevelAssets>,
) {
    info!("Initting level");

    let level = levels.get(&base_level_assets.level).unwrap();
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

fn setup_tiles(
    mut map_ready_event: EventReader<MapReady>,
    query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>,
) {
    for ready in map_ready_event.iter() {
        crate::level::activeatable_tile_setup(query);
        break;
    }
}

fn spawn_objects(
    mut map_ready_event: EventReader<MapReady>,
    mut commands: Commands,
    start_q: Query<&TilePos, With<StartTileTag>>,
    player_base_assets: Res<BasePlayerAssets>,
    player_generated_assets: Res<GeneratedPlayerAssets>,
    mut map: MapQuery,
    map_entity: Query<&Transform, With<Map>>,
) {
    use bevy::sprite::MaterialMesh2dBundle;

    info!("Initing objects");
    for ready in map_ready_event.iter() {
        let map_tf = map_entity.single();
        for pos in start_q.iter() {
            spawn_player(
                &mut commands,
                ready.map_id,
                ready.layer_id,
                (pos.0, pos.1),
                &mut map,
                map_tf,
                &*player_base_assets,
                &*player_generated_assets,
            );
        }
    }

    commands.insert_resource(NextState(GameState::InGame));
}

pub fn setup_states(app: &mut App) {
    AssetLoader::new(GameState::LoadingLevel(LoadingLevelSubstate::LoadingBaseAssets))
        .continue_to_state(GameState::LoadingLevel(LoadingLevelSubstate::SpawningObjects))
        .with_collection::<BasePlayerAssets>()
        .with_collection::<BaseLevelAssets>()
        .init_resource::<GeneratedPlayerAssets>()
        .build(app);

    app
        .add_enter_system(GameState::LoadingLevel(LoadingLevelSubstate::SpawningObjects), spawn_level)
        .add_system(spawn_objects.run_in_state(GameState::LoadingLevel(LoadingLevelSubstate::SpawningObjects)))
        .add_system(setup_tiles.run_in_state(GameState::LoadingLevel(LoadingLevelSubstate::SpawningObjects)));
}
