use bevy::prelude::*;
use bevy_asset_loader::loading_state::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimations;
use iyes_loopless::prelude::*;

use super::{ GameState, jump_to_state };
use crate::level::{ LevelTilesetImages, BaseLevelAssets, queue_level_tileset_images, init_level_resource, spawn_level, tileset_indexing };
use crate::player::{ GeneratedPlayerAssets, BasePlayerAssets, spawn_player };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingLevel {
    LoadingBaseAssets,
    LoadingLevelTiles,
    InittingLevelResources,
    SpawningLevel,
    SpawningPlayer,
    Cleanup,
    Done,
}

pub fn setup_states(app: &mut App) {
    // Loading base assets
    app
        .add_enter_system(LoadingLevel::LoadingBaseAssets, |mut anims: ResMut<CPUTileAnimations>| anims.clear())
        .add_loading_state(LoadingState::new(LoadingLevel::LoadingBaseAssets)
            .continue_to_state(LoadingLevel::LoadingLevelTiles)
            .with_collection::<BasePlayerAssets>()
            .with_collection::<BaseLevelAssets>()
            .init_resource::<GeneratedPlayerAssets>()
        )
        .add_exit_system(LoadingLevel::LoadingBaseAssets, queue_level_tileset_images);

    // Loading level tiles
    app
        .add_loading_state(LoadingState::new(LoadingLevel::LoadingLevelTiles)
            .with_collection::<LevelTilesetImages>()
            .continue_to_state(LoadingLevel::InittingLevelResources)
        );

    // Inititing level resources
    app.add_enter_system_set(
        LoadingLevel::InittingLevelResources, 
        SystemSet::new()
            .with_system(tileset_indexing.chain(init_level_resource))
            .with_system(jump_to_state(LoadingLevel::SpawningLevel))
    );

    // Spawning level
    app.add_enter_system_set(
        LoadingLevel::SpawningLevel,
        SystemSet::new()
            .with_system(spawn_level)
            .with_system(jump_to_state(LoadingLevel::SpawningPlayer))
    );

    // Spawning a player
    app.add_enter_system_set(
        LoadingLevel::SpawningPlayer, 
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
