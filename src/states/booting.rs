use bevy::prelude::*;
use bevy_asset_loader::{ dynamic_asset::*, loading_state::* };
use bevy_pkv::PkvStore;

use crate::{LaunchParams, save::Save};

use super::{ GameState };

fn save_check(mut pkv: ResMut<PkvStore>) {
    if let Err(e) = pkv.get::<Save>("save") {
        info!("Save check returned an error: {e}");
        info!("Inserting the default value");

        // FIXME this creates a user-unfriendly crash
        // FIXME this nukes the broken save. It would be better to back it up
        pkv.set("save", &Save::new()).unwrap();
    }
}

pub fn setup_states(app: &mut App, _params: &LaunchParams) {
    use super::main_menu::MenuAssets;
    use super::splash_screen::ScreenAssets;

    app
        .add_system(save_check.in_schedule(OnEnter(GameState::Booting)))
        .add_loading_state(
            LoadingState::new(GameState::Booting)
            .continue_to_state(GameState::SplashScreen)
        )
        .add_collection_to_loading_state::<_, ScreenAssets>(GameState::Booting)
        .add_collection_to_loading_state::<_, MenuAssets>(GameState::Booting);
}
