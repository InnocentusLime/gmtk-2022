use bevy::prelude::*;
use bevy_asset_loader::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use super::GameState;

use crate::level::*;
use crate::player::PlayerState;

#[derive(AssetCollection)]
pub struct InGameAssets {
    #[asset(path = "maps/test_map.tmx")]
    pub level: Handle<Level>,
    #[asset(path = "tiles/level_tiles.png")]
    pub _atlas: Handle<Image>,
    #[asset(path = "player.png")]
    pub player_texture: Handle<Image>,
}

fn enter() {
    info!("Entered ingame state");
}

fn exit() {
    info!("Exited ingame state");
}

pub fn setup_states(app: &mut App) {
    use iyes_loopless::state::app::StateTransitionStageLabel;
    app
        .add_enter_system(GameState::InGame, enter)
        .add_exit_system(GameState::InGame, exit);
}
