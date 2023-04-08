mod booting;
mod ingame;
mod main_menu;
mod loading;
mod splash_screen;

pub use ingame::GameWorldTag;

use bevy::prelude::*;
use bevy_asset_loader::{ standard_dynamic_asset::*, dynamic_asset::* };
use iyes_loopless::prelude::*;
use loading::LoadingLevel;

use crate::LaunchParams;

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
    // Loading a level
    LoadingLevel,
    // The game
    InGame,
}

pub fn jump_to_state<T: bevy::ecs::schedule::StateData>(state: T) -> impl Fn(Commands) {
    move |mut commands: Commands| commands.insert_resource(NextState(state.clone()))
}

pub fn enter_level(level_path: String, commands: &mut Commands, asset_keys: &mut DynamicAssets) {
    asset_keys.register_asset("map", Box::new(StandardDynamicAsset::File { path: level_path }));
    commands.insert_resource(NextState(GameState::LoadingLevel));
    commands.insert_resource(NextState(LoadingLevel::BaseAssets));
}

pub fn setup_states(app: &mut App, params: &LaunchParams) {
    app.add_loopless_state(GameState::Booting);
    app.add_loopless_state(LoadingLevel::Done);

    booting::setup_states(app, params);
    splash_screen::setup_states(app, params);
    main_menu::setup_states(app, params);
    loading::setup_states(app, params);

    ingame::setup_states(app, params);
}
