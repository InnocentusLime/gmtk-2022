use bevy::prelude::*;
use bevy_asset_loader::{ dynamic_asset::*, loading_state::* };

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

    app.add_system(
        exit.in_schedule(OnExit(GameState::Booting))
    );

    match params.level_file {
        Some(testing_level) => {
            // let path = testing_level.to_owned();
            // app.add_system(
            //     move |mut commands: Commands, mut asset_keys: ResMut<DynamicAssets>| {
            //         info!("Entered booting state");
            //         enter_level(path.clone(), &mut commands, &mut asset_keys);
            //     }
            //     .in_schedule(OnEnter(GameState::Booting))
            // );
        },
        None => {
            app
                .add_system(
                    enter_normal
                    .in_schedule(OnEnter(GameState::Booting))
                )
                .add_loading_state(
                    LoadingState::new(GameState::Booting)
                    .continue_to_state(GameState::SplashScreen)
                )
                .add_system(
                    exit
                    .in_schedule(OnExit(GameState::Booting))
                )
                .add_collection_to_loading_state::<_, ScreenAssets>(GameState::Booting)
                .add_collection_to_loading_state::<_, MenuAssets>(GameState::Booting);
        },
    }
}
