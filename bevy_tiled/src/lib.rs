//! This crate provides a few useful tools that the game uses when loading
//! its levels from files.

pub extern crate tiled;

pub mod tiled_ext;
pub mod tiled_map_asset;
pub mod map_scheme;

pub use tiled_ext::*;
pub use tiled_map_asset::*;
//pub use map_scheme::*;

use bevy::prelude::*;

#[derive(Default)]
pub struct TiledPlugin;

impl Plugin for TiledPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_asset::<TiledMap>()
            .add_asset_loader(TiledMapLoader);
    }
} 
