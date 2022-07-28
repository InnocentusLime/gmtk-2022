use bevy_asset_loader::*;
use bevy::prelude::*;

use super::Level;

#[derive(AssetCollection)]
pub struct BaseLevelAssets {
    #[asset(key = "level")]
    pub level: Handle<Level>,
}
