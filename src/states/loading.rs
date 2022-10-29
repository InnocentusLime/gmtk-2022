use bevy::prelude::*;
use bevy_asset_loader::loading_state::*;
use iyes_loopless::prelude::*;

use super::{ GameState, jump_to_state };
use bevy_tiled::tileset_indexing;
use crate::level::{ LevelTilesetImages, BaseLevelAssets, queue_level_tileset_images, spawn_level, get_level_map };
use crate::player::{ GeneratedPlayerAssets, BasePlayerAssets, spawn_player };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingLevel {
    BaseAssets,
    LevelTiles,
    LevelEntity,
    PlayerEntity,
    Cleanup,
    Done,
}

pub fn setup_states(app: &mut App) {
    // Loading base assets
    app
        .add_loading_state(LoadingState::new(LoadingLevel::BaseAssets)
            .with_collection::<BasePlayerAssets>()
            .with_collection::<BaseLevelAssets>()
            .init_resource::<GeneratedPlayerAssets>()
            .continue_to_state(LoadingLevel::LevelTiles)
        )
        .add_exit_system(LoadingLevel::BaseAssets, queue_level_tileset_images);

    // Loading level tiles
    app
        .add_loading_state(LoadingState::new(LoadingLevel::LevelTiles)
            .with_collection::<LevelTilesetImages>()
            .continue_to_state(LoadingLevel::LevelEntity)
        );

    // Inititing level resources
    app.add_enter_system_set(
        LoadingLevel::LevelEntity, 
        SystemSet::new()
            .with_system(
                get_level_map.chain(tileset_indexing).chain(spawn_level)
            )
            .with_system(jump_to_state(LoadingLevel::Cleanup))
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
