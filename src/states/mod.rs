mod booting;
mod ingame;
mod main_menu;
mod loading;
mod splash_screen;

pub use ingame::GameWorldTag;

use bevy::prelude::*;
use bevy_asset_loader::{ standard_dynamic_asset::*, dynamic_asset::* };
use loading::LoadingLevel;

use crate::LaunchParams;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    // The booting state loads the assets for
    // splash screen and main menu.
    #[default]
    Booting,
    // Shows a sequence of logos related to the
    // game
    SplashScreen,
    // The main menu
    MainMenu,
    // Loading a level
    LoadingLevel,
    // The game
    InGame,
}

pub fn jump_to_state<T: States>(state: T) -> impl Fn(ResMut<NextState<T>>) {
    move |mut next| next.0 = Some(state.clone())
}

pub fn enter_level(
    level_path: String,
    asset_keys: &mut DynamicAssets,
    load_lvl_st: &mut NextState<LoadingLevel>,
    game_st: &mut NextState<GameState>,
) {
    asset_keys.register_asset("map", Box::new(StandardDynamicAsset::File { path: level_path }));
    game_st.0 = Some(GameState::LoadingLevel);
    load_lvl_st.0 = Some(LoadingLevel::BaseAssets);
}

pub fn setup_states(app: &mut App, params: &LaunchParams) {
    app.add_state::<GameState>();
    app.add_state::<LoadingLevel>();

    booting::setup_states(app, params);
    splash_screen::setup_states(app, params);
    main_menu::setup_states(app, params);
    loading::setup_states(app, params);

    ingame::setup_states(app, params);
}
