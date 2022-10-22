use bevy_asset_loader::asset_collection::*;
use bevy::prelude::*;
use bevy_tiled::TiledMap;

#[derive(AssetCollection)]
pub struct BaseLevelAssets {
    #[asset(key = "map")]
    pub map: Handle<TiledMap>,
}

#[derive(AssetCollection)]
pub struct LevelTilesetImages {
    #[asset(key = "tileset_images", collection(typed))]
    pub images: Vec<Handle<TextureAtlas>>,
}
