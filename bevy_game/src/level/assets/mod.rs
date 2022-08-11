mod tileset_indexing;

use std::io::BufReader;
use std::path::PathBuf;

use super::LevelTilesetImages;
use bevy::asset::{ AssetServer, AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset };
use bevy_asset_loader::dynamic_asset::{ DynamicAsset, DynamicAssetType };
use bevy::prelude::*;
use bevy::reflect::TypeUuid;

pub use tileset_indexing::TilesetIndexing;

/// Encodes the types for the tilset
#[derive(Clone, Debug)]
pub enum TiledTileset {
    /// The tilset is a single image
    Image(AssetPath<'static>),
    /// Each tile has an individual image. So we keep
    /// a mapping from tile ID to paths.
    ImageCollection(Vec<(u32, AssetPath<'static>)>),
}

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct TiledMap {
    pub map: tiled::Map,
    pub tilesets: Vec<(Vec2, TiledTileset)>,
}

impl TiledMap {
    pub fn get_tileset_dynamic_asset(&self) -> impl DynamicAsset { 
        TilesetsFromTiled(self.tilesets.clone()) 
    }
}

#[derive(Debug)]
struct TilesetsFromTiled(Vec<(Vec2, TiledTileset)>);

impl DynamicAsset for TilesetsFromTiled {
    fn load(&self, asset_server: &AssetServer) -> Vec<HandleUntyped> {
        let mut res = Vec::new();

        self.0.iter()
            .for_each(|(_, t)| match t {
                TiledTileset::Image(p) => res.push(asset_server.load_untyped(p.to_owned())),
                TiledTileset::ImageCollection(c) => res.extend(
                    c.iter().map(|(_, p)| asset_server.load_untyped(p.to_owned()))
                ),
            });

        info!("Compiled list of tiles");

        res
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        info!("Building tilesets");
        let cell = world.cell();
        let mut images = cell
            .get_resource_mut::<Assets<Image>>()
            .expect("Failed to get image asset container");
        let mut atlases = cell
            .get_resource_mut::<Assets<TextureAtlas>>()
            .expect("Failed to get image asset container");
        let res = self.0.iter()
            .map(|(tile_size, tileset)| match tileset {
                TiledTileset::Image(p) => {
                    let image_handle = images.get_handle(p.to_owned());
                    let image = images.get(&image_handle).expect("Image should be loaded");
                    let image_size = image.size();

                    Ok(atlases.add(TextureAtlas::from_grid(
                        image_handle, 
                        *tile_size, 
                        image_size.x as usize / tile_size.x as usize,
                        image_size.y as usize / tile_size.y as usize,
                    )).clone_untyped())
                },
                TiledTileset::ImageCollection(c) => {
                    let tileset_size = Vec2::new(tile_size.x, tile_size.y * c.len() as f32);
                    let mut builder = TextureAtlasBuilder::default().initial_size(tileset_size).max_size(tileset_size);

                    c.iter()
                        .map(|(_, p)| images.get_handle(p.to_owned()))
                        .for_each(|handle| builder.add_texture(
                            handle.clone(),
                            images.get(&handle).expect("Image should be loaded")
                        ));
            
                    let result = builder.finish(&mut *images)?;
                    
                    Ok(atlases.add(result).clone_untyped())
                },
            })
            .collect::<Result<_, anyhow::Error>>()?;

        Ok(DynamicAssetType::Collection(res))
    }
}

#[derive(Clone, Copy, Default)]
pub struct TiledMapLoader;

impl AssetLoader for TiledMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        // NOTE this is a workround, because of `tiled`'s bad compatability
        // with `bevy`. It uses `fs::File` to load tileset, which ends up 
        // peeking into the crate root, rather into asset folder.
        fn asset_dir_root() -> PathBuf {
            #[cfg(target_arch = "x86_64")]
            return bevy::asset::FileAssetIo::get_base_path();

            #[cfg(target_arch = "wasm32")]
            return PathBuf::new();
        }

        Box::pin(async move {
            let mut loader = tiled::Loader::new();
            
            let root = asset_dir_root().join("assets");
            // FIXME `tiled` loads dependencies using Rust's traditional file IO. That's very bad
            // news for the WASM build. 
            let map = loader.load_tmx_map_from(BufReader::new(bytes), &root.join(load_context.path()))?;
            let fix_asset_path = |x: &PathBuf| AssetPath::new(x.strip_prefix(&root).unwrap().to_path_buf(), None);
            let mut tilesets = Vec::new();

            for tileset in map.tilesets() {
                let tile_size = Vec2::new(tileset.tile_width as f32, tileset.tile_height as f32);
                let tileset = match tileset.image.as_ref() {
                    Some(image) => {
                        let asset_path = fix_asset_path(&image.source);
                        
                        TiledTileset::Image(asset_path)
                    },
                    None => {
                        let asset_paths: Vec<(u32, AssetPath<'static>)> = tileset.tiles()
                            .filter_map(|(tile_id, tile)|
                                tile.image.as_ref().map(|x| (tile_id, fix_asset_path(&x.source)))
                            )
                            .collect();

                        TiledTileset::ImageCollection(asset_paths)
                    },
                };
                
                tilesets.push((tile_size, tileset));
            }

            let loaded_asset = LoadedAsset::new(TiledMap {
                map, tilesets,
            });

            load_context.set_default_asset(loaded_asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] { &["tmx"] }
}
