use bevy::prelude::*;
use iyes_loopless::prelude::*;

use super::GameState;

use crate::states::main_menu::MenuAssets;
use crate::save::Save;
use crate::level_info::LevelInfo;
use crate::player::PlayerModification;
use crate::app::{ GameplayCamera, MenuCamera };

struct LevelCompleteCountdown(Timer);

fn enter() {
    info!("Entered ingame state");
}

fn death_system(
    mut commands: Commands,
    mut events: EventReader<PlayerModification>,
) {
    for ev in events.iter() {
        match ev {
            PlayerModification::Kill => {
                info!("You are dead");
                commands.insert_resource(NextState(GameState::MainMenu));
            },
            _ => (),
        }
    }
}

fn beat_system(
    mut commands: Commands,
    mut events: EventReader<PlayerModification>,
    mut save: ResMut<Save>,
    audio: Res<Audio>,
    menu_assets: Res<MenuAssets>,
    level_infos: Res<Assets<LevelInfo>>,
) {
    for ev in events.iter() {
        match ev {
            PlayerModification::Escape => {
                info!("You win");
                let level_info = level_infos.get(&menu_assets.level_info).unwrap();
                //audio.play(assets.complete_sound.clone());
         
                save.register_level_complete(&*level_info);
                // TODO retry?
                match save.save() {
                    Ok(()) => (),
                    Err(e) => error!("Error recording save: {}\nThe progress will be lost.", e),
                }
                commands.insert_resource(LevelCompleteCountdown(Timer::from_seconds(2.0f32, false)));
            },
            _ => (),
        }
    }
}

fn level_complete_system(
    mut commands: Commands,
    mut timer: Option<ResMut<LevelCompleteCountdown>>,
    time: Res<Time>,
) {
    if let Some(timer) = timer.as_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.remove_resource::<LevelCompleteCountdown>();
            commands.insert_resource(NextState(GameState::MainMenu));
        }
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
    app
        .add_enter_system(GameState::InGame, enter)
        .add_system(level_complete_system.run_in_state(GameState::InGame))
        .add_system(beat_system.run_in_state(GameState::InGame))
        .add_system(death_system.run_in_state(GameState::InGame))
        .add_exit_system(GameState::InGame, exit);
}
