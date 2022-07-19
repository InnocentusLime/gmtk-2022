use bevy::prelude::*;
use bevy_asset_loader::*;
use iyes_loopless::prelude::*;

use super::GameState;
use super::ingame::{ InGameAssets, PlayerGraphicsResources };

fn enter() {
    info!("Entered loading state");
}

fn exit() {
    info!("Exited loading state");
}

pub fn setup_states(app: &mut App) {
    app
        .add_enter_system(GameState::Loading, enter)
        .add_exit_system(GameState::Loading, exit);

    AssetLoader::new(GameState::Loading)
        .continue_to_state(GameState::Spawning)
        .with_collection::<InGameAssets>()
        .init_resource::<PlayerGraphicsResources>()
        .build(app);
}
