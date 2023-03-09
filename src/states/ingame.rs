use bevy::prelude::*;
use bevy_pkv::PkvStore;
use iyes_loopless::prelude::*;

use super::GameState;

use crate::states::main_menu::MenuAssets;
use crate::save::Save;
use crate::level_info::LevelInfo;
use crate::player::{ PlayerTag };
use crate::tile::TileEvent;
use crate::{GameplayCamera, LaunchParams};

#[derive(Resource)]
struct LevelCompleteCountdown(Timer);

fn enter() {
    info!("Entered ingame state");
}

fn death_system_normal(mut commands: Commands, player_q: Query<(), With<PlayerTag>>) {
    if player_q.is_empty() {
        info!("You are dead");
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn death_system_testing_level(mut writer: EventWriter<bevy::app::AppExit>, player_q: Query<(), With<PlayerTag>>) {
    if player_q.is_empty() {
        info!("You are dead");
        writer.send(bevy::app::AppExit);
    }
}

fn beat_system(
    mut commands: Commands,
    mut tile_events: EventReader<TileEvent>,
) {
    if tile_events.iter().filter_map(|x| match x {
            TileEvent::ExitReached => Some(()),
            _ => None,
        }).next().is_some()
    {
        info!("You win");
        commands.insert_resource(LevelCompleteCountdown(Timer::from_seconds(2.0f32, TimerMode::Once)));
    }
}

fn level_complete_system_normal(
    mut commands: Commands,
    mut timer: Option<ResMut<LevelCompleteCountdown>>,
    mut save: ResMut<Save>,
    menu_assets: Res<MenuAssets>,
    level_infos: Res<Assets<LevelInfo>>,
    time: Res<Time>,
    mut pkv: ResMut<PkvStore>,
) {
    if let Some(timer) = timer.as_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let level_info = level_infos.get(&menu_assets.level_info).unwrap();

            save.register_level_complete(level_info);

            // TODO retry?
            match pkv.set("save", &*save) {
                Ok(()) => (),
                Err(e) => error!("Error recording save: {}\nThe progress will be lost.", e),
            }

            commands.remove_resource::<LevelCompleteCountdown>();
            commands.insert_resource(NextState(GameState::MainMenu));
        }
    }
}

fn level_complete_system_testing_level(
    mut writer: EventWriter<bevy::app::AppExit>,
    mut timer: Option<ResMut<LevelCompleteCountdown>>,
    time: Res<Time>,
) {
    if let Some(timer) = timer.as_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            writer.send(bevy::app::AppExit);
        }
    }
}

fn exit(
    mut commands: Commands,
    mut cam: Query<&mut Transform, With<GameplayCamera>>,
    to_del: Query<Entity, Without<GameplayCamera>>,
) {
    info!("Exited ingame state");
    for mut tf in cam.iter_mut() { tf.translation = Vec3::new(0.0f32, 0.0f32, 50.0f32); }

    for e in to_del.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_states(app: &mut App, params: &LaunchParams) {
    app
        .add_enter_system(GameState::InGame, enter)
        .add_system(beat_system.run_in_state(GameState::InGame))
        .add_exit_system(GameState::InGame, exit);

    if params.level_file.is_some() {
        app
            .add_system(level_complete_system_testing_level.run_in_state(GameState::InGame))
            .add_system(death_system_testing_level.run_in_state(GameState::InGame));
    } else {
        app
            .add_system(level_complete_system_normal.run_in_state(GameState::InGame))
            .add_system(death_system_normal.run_in_state(GameState::InGame));
    }
}
