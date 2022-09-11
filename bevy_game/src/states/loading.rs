use bevy::prelude::*;
use bevy_asset_loader::loading_state::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimations;
use iyes_loopless::prelude::*;

use super::{ GameState, jump_to_state };
use bevy_tiled::tileset_indexing;
use crate::level::{ LevelTilesetImages, BaseLevelAssets, queue_level_tileset_images, init_level_resource, spawn_level, get_level_map };
use crate::player::{ GeneratedPlayerAssets, BasePlayerAssets, spawn_player };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingLevel {
    BaseAssets,
    LevelTiles,
    LevelResources,
    LevelEntity,
    PlayerEntity,
    Cleanup,
    Done,
}

pub fn setup_states(app: &mut App) {
    // Loading base assets
    app
        .add_enter_system(LoadingLevel::BaseAssets, |mut anims: ResMut<CPUTileAnimations>| anims.clear())
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
            .continue_to_state(LoadingLevel::LevelResources)
        );

    // Inititing level resources
    app.add_enter_system_set(
        LoadingLevel::LevelResources, 
        SystemSet::new()
            .with_system(
                get_level_map.chain(tileset_indexing).chain(init_level_resource)
            )
            .with_system(jump_to_state(LoadingLevel::LevelEntity))
    );

    // Spawning level
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
