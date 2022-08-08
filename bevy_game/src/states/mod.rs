mod booting;
mod ingame;
mod main_menu;
mod loading;
mod splash_screen;

use bevy::prelude::*;
use bevy_asset_loader::{ standard_dynamic_asset::*, dynamic_asset::* };
use iyes_loopless::prelude::*;
use loading::LoadingLevelSubstate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    // The booting state loads the assets for 
    // splash screen and main menu.
    Booting,
    // Shows a sequence of logos related to the
    // game
    SplashScreen,
    // The main menu
    MainMenu,
    // The state which loads assets for InGame state
    LoadingLevel(loading::LoadingLevelSubstate),
    // The game
    InGame,
}

pub fn enter_level(level_path: String, commands: &mut Commands, asset_keys: &mut DynamicAssets) {
    asset_keys.register_asset("level", Box::new(StandardDynamicAsset::File { path: level_path }));
    commands.insert_resource(NextState(GameState::LoadingLevel(LoadingLevelSubstate::LoadingBaseAssets)));
}

pub fn setup_states(app: &mut App, testing_level: Option<&str>) {
    app.add_loopless_state(GameState::Booting);

    booting::setup_states(app, testing_level);
    splash_screen::setup_states(app);
    main_menu::setup_states(app);
    loading::setup_states(app);

    ingame::setup_states(app, testing_level.is_some());
}
