use std::collections::HashMap;
use std::io::BufReader;

use bevy::asset::{ AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset };
use bevy::prelude::*;

use super::asset::*;

#[derive(Clone, Copy, Default)]
pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            use std::path::Path; 

            let mut loader = tiled::Loader::new();
            // TODO not entirely correct
            let map = loader.load_tmx_map_from(BufReader::new(bytes), &Path::new("assets/").join(load_context.path()))?;

            let mut dependencies = Vec::new();
            let mut handles = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                // TODO mhm
                let tile_path = tileset.image.as_ref().unwrap().source.strip_prefix("assets/").unwrap();
                let asset_path = AssetPath::new(tile_path.to_path_buf(), None);
                let texture: Handle<Image> = load_context.get_handle(asset_path.clone());

                handles.insert(tileset_index, texture.clone());

                dependencies.push(asset_path);
            }

            let loaded_asset = LoadedAsset::new(Level::new(
                map, handles
            )).with_dependencies(dependencies);

            load_context.set_default_asset(loaded_asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] { &["tmx"] }
}
