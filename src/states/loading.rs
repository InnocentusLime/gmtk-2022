use bevy::prelude::*;
use bevy_asset_loader::loading_state::*;
use iyes_loopless::prelude::*;

use super::ingame::GameWorldTag;
use super::{ GameState, jump_to_state };
use crate::LaunchParams;
use crate::level::{ spawn_level };
use crate::player::{ GeneratedPlayerAssets, BasePlayerAssets, spawn_player };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingLevel {
    BaseAssets,
    GameWorld,
    LevelEntity,
    PlayerEntity,
    Cleanup,
    Done,
}

pub fn spawn_game_world(mut commands: Commands) {
    commands.spawn((Name::new("GameWorld"), GameWorldTag));
}

pub fn setup_states(app: &mut App, _params: &LaunchParams) {
    // Loading base assets
    app
        .add_loading_state(LoadingState::new(LoadingLevel::BaseAssets)
            .with_collection::<BasePlayerAssets>()
            .init_resource::<GeneratedPlayerAssets>()
            .continue_to_state(LoadingLevel::GameWorld)
        );

    // Spawning game world node
    app.add_enter_system_set(
        LoadingLevel::GameWorld,
        SystemSet::new()
            .with_system(spawn_game_world)
            .with_system(jump_to_state(LoadingLevel::LevelEntity))
    );

    // Inititing level resources
    app.add_enter_system_set(
        LoadingLevel::LevelEntity,
        SystemSet::new()
            .with_system(spawn_level)
            .with_system(jump_to_state(LoadingLevel::PlayerEntity))
    );

    // Spawning a player
    app.add_enter_system_set(
        LoadingLevel::PlayerEntity,
        SystemSet::new()
            .with_system(spawn_player)
            .with_system(jump_to_state(LoadingLevel::Cleanup))
    );

    // Cleanup
    app
        .add_enter_system_set(
            LoadingLevel::Cleanup,
            SystemSet::new()
                .with_system(|server: Res<AssetServer>| {
                    server.mark_unused_assets();
                    server.free_unused_assets();
                })
                .with_system(jump_to_state(GameState::InGame))
                .with_system(jump_to_state(LoadingLevel::Done))
        );
}
