use bevy::prelude::*;
use bevy_asset_loader::loading_state::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimations;
use iyes_loopless::prelude::*;

use super::GameState;
use crate::level::{ LevelTilesetImages, BaseLevelAssets, queue_level_tileset_images, spawn_level, prepare_level_tileset_images };
use crate::player::{ GeneratedPlayerAssets, BasePlayerAssets, spawn_player };
use crate::tile::activeatable_tile_setup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingLevelSubstate {
    LoadingBaseAssets,
    LoadingLevelTiles,
    SpawningLevel,
    SpawningPlayer,
    InittingTiles,
    Cleanup,
}

pub fn setup_states(app: &mut App) {
    // Loading base assets
    app
        .add_enter_system(
            GameState::LoadingLevel(LoadingLevelSubstate::LoadingBaseAssets),
            |mut anims: ResMut<CPUTileAnimations>| anims.clear()
        )
        .add_loading_state(LoadingState::new(GameState::LoadingLevel(LoadingLevelSubstate::LoadingBaseAssets))
            .continue_to_state(GameState::LoadingLevel(LoadingLevelSubstate::LoadingLevelTiles))
            .with_collection::<BasePlayerAssets>()
            .with_collection::<BaseLevelAssets>()
            .init_resource::<GeneratedPlayerAssets>()
        )
        .add_exit_system(
            GameState::LoadingLevel(LoadingLevelSubstate::LoadingBaseAssets),
            queue_level_tileset_images
        );

    // Loading level tiles
    app
        .add_loading_state(LoadingState::new(GameState::LoadingLevel(LoadingLevelSubstate::LoadingLevelTiles))
            .with_collection::<LevelTilesetImages>()
            .continue_to_state(GameState::LoadingLevel(LoadingLevelSubstate::SpawningLevel))
        )
        .add_exit_system(
            GameState::LoadingLevel(LoadingLevelSubstate::LoadingLevelTiles),
            prepare_level_tileset_images
        );
   
    // Spawning level
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::SpawningLevel),
        |mut commands: Commands| commands.insert_resource(NextState(
            GameState::LoadingLevel(
                LoadingLevelSubstate::SpawningPlayer
            )
        ))
    );
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::SpawningLevel),
        spawn_level
    );

    // Spawning a player
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::SpawningPlayer), 
        |mut commands: Commands| commands.insert_resource(NextState(
            GameState::LoadingLevel(
                LoadingLevelSubstate::InittingTiles
            )
        ))
    );
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::SpawningPlayer), 
        spawn_player
    );

    // Initting tiles
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::InittingTiles), 
        |mut commands: Commands| commands.insert_resource(NextState(
            GameState::LoadingLevel(
                LoadingLevelSubstate::Cleanup
            )
        ))
    );
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::InittingTiles), 
        activeatable_tile_setup
    );

    // Cleanup
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::Cleanup),
        |server: Res<AssetServer>| {
            server.mark_unused_assets();
            server.free_unused_assets();
        }
    );
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::Cleanup),
        |mut commands: Commands| commands.insert_resource(NextState(
            GameState::InGame
        ))
    );
}
