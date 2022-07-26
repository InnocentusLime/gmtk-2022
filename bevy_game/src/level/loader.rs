use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;

use bevy::asset::{ AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset };
//use bevy_ecs_tilemap::prelude::TileAtlasBuilder;
use bevy::prelude::*;

use super::asset::*;

#[derive(Clone, Copy, Default)]
pub struct TiledLoader;

fn asset_dir_root() -> PathBuf {
    #[cfg(target_arch = "x86_64")]
    return bevy::asset::FileAssetIo::get_root_path();

    #[cfg(target_arch = "wasm32")]
    return PathBuf::new();
}

impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut loader = tiled::Loader::new();
            // TODO not entirely correct
            let root = asset_dir_root().join("assets");
            let map = loader.load_tmx_map_from(BufReader::new(bytes), &root.join(load_context.path()))?;

            let mut dependencies = Vec::new();
            let mut handles = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                match tileset.image.as_ref() {
                    Some(image) => {
                        //warn!("Atlased tileset is not recommended. Please split your tileset into many tile files.");
                        let tile_path = image.source.strip_prefix(&root).unwrap();
                        let asset_path = AssetPath::new(tile_path.to_path_buf(), None);
                        let texture: Handle<Image> = load_context.get_handle(asset_path.clone());
                        handles.insert(tileset_index, texture.clone());
                        dependencies.push(asset_path);
                    },
                    None => {
                        /*
                        let mut atlas_builder = TileAtlasBuilder::new(
                            Vec2::new(map.tile_width, map.tile_height)
                        );
                        // Hmmm
                        for (_, tile) in tileset.tiles {}
                        */
                        todo!()
                    },
                }
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
