use bevy::prelude::*;
use bevy_asset_loader::*;
use iyes_loopless::prelude::*;

use super::GameState;

fn enter() {
    info!("Entered ingame state");
}

fn exit() {
    info!("Exited ingame state");
}

pub fn setup_states(app: &mut App) {
    app
        .add_enter_system(GameState::InGame, enter)
        .add_exit_system(GameState::InGame, exit);
}
