use bevy::prelude::*;
use bevy_asset_loader::*;
use iyes_loopless::prelude::*;

use super::GameState;
use crate::level::{ BaseLevelAssets, spawn_level };
use crate::player::{ GeneratedPlayerAssets, BasePlayerAssets, spawn_player };
use crate::tile::activeatable_tile_setup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingLevelSubstate {
    LoadingBaseAssets,
    SpawningLevel,
    SpawningPlayer,
    InittingTiles,
}

pub fn setup_states(app: &mut App) {
    AssetLoader::new(GameState::LoadingLevel(LoadingLevelSubstate::LoadingBaseAssets))
        .continue_to_state(GameState::LoadingLevel(LoadingLevelSubstate::SpawningLevel))
        .with_collection::<BasePlayerAssets>()
        .with_collection::<BaseLevelAssets>()
        .init_resource::<GeneratedPlayerAssets>()
        .build(app);

    /*
    app.add_enter_system_set(
        GameState::LoadingLevel(LoadingLevelSubstate::SpawningLevel),
        SystemSet::new()
            .with_system(|mut commands: Commands| commands.insert_resource(NextState(
                GameState::LoadingLevel(
                    LoadingLevelSubstate::SpawningPlayer
                )
            )))
            .with_system(spawn_level)
    );

    app.add_enter_system_set(
        GameState::LoadingLevel(LoadingLevelSubstate::SpawningPlayer), 
        SystemSet::new()
            .with_system(|mut commands: Commands| commands.insert_resource(NextState(
                GameState::LoadingLevel(
                    LoadingLevelSubstate::InittingTiles
                )
            )))
            .with_system(spawn_player)
    );

    app.add_enter_system_set(
        GameState::LoadingLevel(LoadingLevelSubstate::InittingTiles), 
        SystemSet::new()
            .with_system(|mut commands: Commands| commands.insert_resource(NextState(
                GameState::InGame
            )))
            .with_system(activeatable_tile_setup)
    );
    */
   
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
            GameState::InGame
        ))
    );
    app.add_enter_system(
        GameState::LoadingLevel(LoadingLevelSubstate::InittingTiles), 
        activeatable_tile_setup
    );
}
