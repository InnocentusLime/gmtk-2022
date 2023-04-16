use bevy::prelude::*;
use bevy_asset_loader::loading_state::*;

use super::GameState;
use crate::LaunchParams;
use crate::player::{ GeneratedPlayerAssets, BasePlayerAssets };

// #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
// pub enum LoadingLevel {
//     BaseAssets,
//     LevelEntity,
//     PlayerEntity,
//     Cleanup,
//     #[default]
//     Done,
// }

pub fn setup_states(app: &mut App, _params: &LaunchParams) {
    // Loading base assets
    app
        .add_loading_state(
            LoadingState::new(GameState::LoadingLevel)
            .continue_to_state(GameState::InGame)
        )
        .add_collection_to_loading_state::<_, BasePlayerAssets>(GameState::LoadingLevel)
        .init_resource_after_loading_state::<_, GeneratedPlayerAssets>(GameState::LoadingLevel);
}
