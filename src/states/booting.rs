use bevy::prelude::*;
use bevy_asset_loader::{ dynamic_asset::*, loading_state::* };
use iyes_loopless::prelude::*;

use crate::LaunchParams;

use super::{ GameState, enter_level };

fn enter_normal() {
    info!("Entered booting state");
}

fn exit() {
    info!("Exited booting state");
}

pub fn setup_states(app: &mut App, params: &LaunchParams) {
    use super::main_menu::MenuAssets;
    use super::splash_screen::ScreenAssets;

    app.add_exit_system(GameState::Booting, exit);

    match params.level_file {
        Some(testing_level) => {
            let path = testing_level.to_owned();
            app.add_enter_system(
                GameState::Booting,
                move |mut commands: Commands, mut asset_keys: ResMut<DynamicAssets>| {
                    info!("Entered booting state");
                    enter_level(path.clone(), &mut commands, &mut asset_keys);
                },
            );
        },
        None => {
            app
                .add_enter_system(GameState::Booting, enter_normal)
                .add_loading_state(
                    LoadingState::new(GameState::Booting)
                    .continue_to_state(GameState::SplashScreen)
                    .with_collection::<ScreenAssets>()
                    .with_collection::<MenuAssets>()
                );
        },
    }
}
