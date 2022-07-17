use bevy::prelude::*;
use bevy_asset_loader::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use super::GameState;

use crate::level::*;
use crate::app::{ GameplayCamera, MenuCamera };
use crate::special_tiles::{ EndReached, DangerReached };

#[derive(AssetCollection)]
pub struct InGameAssets {
    #[asset(path = "maps/test_map.tmx")]
    pub level: Handle<Level>,
    #[asset(path = "Dice.glb")]
    pub player_gltf: Handle<bevy::gltf::Gltf>,
}

fn enter() {
    info!("Entered ingame state");
}

fn death_system(
    mut commands: Commands,
    mut events: EventReader<DangerReached>
) {
    for ev in events.iter() {
        info!("You are dead");
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn beat_system(
    mut commands: Commands,
    mut events: EventReader<EndReached>
) {
    for ev in events.iter() {
        info!("You win");
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn exit(
    mut commands: Commands,
    mut cam: Query<&mut Transform, With<GameplayCamera>>,
    to_del: Query<Entity, (Without<GameplayCamera>, Without<MenuCamera>)>,
) {
    info!("Exited ingame state");
    for mut tf in cam.iter_mut() { tf.translation = Vec3::new(0.0f32, 0.0f32, 50.0f32); }

    for e in to_del.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_states(app: &mut App) {
    use iyes_loopless::state::app::StateTransitionStageLabel;
    app
        .add_enter_system(GameState::InGame, enter)
        .add_system(beat_system.run_in_state(GameState::InGame))
        .add_system(death_system.run_in_state(GameState::InGame))
        .add_exit_system(GameState::InGame, exit);
}
