use bevy::prelude::*;
use bevy_asset_loader::loading_state::*;

use super::{ GameState, jump_to_state };
use crate::LaunchParams;
use crate::level::{ spawn_level };
use crate::player::{ GeneratedPlayerAssets, BasePlayerAssets, spawn_player, PlayerTag };

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum LoadingLevel {
    BaseAssets,
    LevelEntity,
    PlayerEntity,
    Cleanup,
    #[default]
    Done,
}

pub fn setup_states(app: &mut App, _params: &LaunchParams) {
    // Loading base assets
    app
        .add_loading_state(
            LoadingState::new(GameState::LoadingLevel)
            .continue_to_state(GameState::InGame)
        )
        .add_collection_to_loading_state::<_, BasePlayerAssets>(GameState::LoadingLevel);
        //.init_resource_after_loading_state::<_, TestProbe>(LoadingLevel::BaseAssets)
        //.init_resource_after_loading_state::<_, GeneratedPlayerAssets>(LoadingLevel::BaseAssets);

    // Inititing level resources
    app.add_systems(
        (spawn_level, jump_to_state(LoadingLevel::PlayerEntity))
        .in_schedule(OnEnter(LoadingLevel::LevelEntity))
    );

    // Spawning a player
    app.add_systems(
        (
            spawn_player, //.run_if(resource_added::<GeneratedPlayerAssets>()),
            jump_to_state(LoadingLevel::Cleanup).run_if(any_with_component::<PlayerTag>())
        ).distributive_run_if(in_state(LoadingLevel::PlayerEntity))
    );

    // Cleanup
    let cleanup = |server: Res<AssetServer>| {
        server.mark_unused_assets();
        server.free_unused_assets();
    };
    app.add_systems(
        (cleanup, jump_to_state(GameState::InGame), jump_to_state(LoadingLevel::Done))
        .in_schedule(OnEnter(LoadingLevel::Cleanup))
    );
}
