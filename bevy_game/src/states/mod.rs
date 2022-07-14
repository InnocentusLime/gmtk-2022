mod booting;
mod ingame;
mod main_menu;
mod splash_screen;

use bevy::prelude::*;
use iyes_loopless::prelude::*;

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
    // The game
    InGame,
}

pub fn setup_states(app: &mut App) {
    app
        .add_loopless_state(GameState::Booting);

    booting::setup_states(app);
    splash_screen::setup_states(app);
    main_menu::setup_states(app);
    ingame::setup_states(app);
}
