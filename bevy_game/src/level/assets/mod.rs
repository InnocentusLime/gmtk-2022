mod tileset_type;
mod tileset_indexing;

use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;

use bevy::asset::{ AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset };
use bevy_ecs_tilemap::prelude::TileAtlasBuilder;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;

pub use tileset_type::TilesetType;
pub use tileset_indexing::TilesetIndexing;

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct Level {
    pub map: tiled::Map,
    pub tileset_indexing: HashMap<usize, TilesetIndexing>,
    pub tilesets: HashMap<usize, TilesetType>,
}

impl Level {
    pub fn new(map: tiled::Map, tilesets: HashMap<usize, TilesetType>) -> Self {
        Level { 
            map, 
            tilesets,
            tileset_indexing: HashMap::new(),
        }
    }

    pub fn find_geometry_layer(&self) -> Option<u16> {
        self.map.layers().enumerate().find(|(_, x)| x.name == "geometry").map(|(x, _)| x as u16)
    }

    pub fn get_tile_texture(&self, tileset: usize, tile: u32) -> u16 {
        self.tileset_indexing[&tileset].dispatch(tile)
    }

    pub fn get_used_images(&self) -> Vec<String> {
        let mut res = Vec::new();

        for (_, v) in &self.tilesets {
            match v {
                TilesetType::Ready(_) => (),
                TilesetType::ImageAtlas { image, .. } => {
                    assert!(image.label().is_none());
                    res.push(image.path().to_str().unwrap().to_owned());
                },
                TilesetType::ImageCollection { collection, .. } => res.extend(
                    collection.iter()
                    .map(|(_, x)| {
                        assert!(x.label().is_none());
                        x.path().to_str().unwrap().to_owned()
                    })
                ),
            }
        }

        res
    }
    
    /// Ensures that `self` is `FullImage`
    pub fn prepare_tilesets(&mut self, textures: &mut Assets<Image>) {
        for (k, v) in &mut self.tilesets {
            match v {
                TilesetType::Ready(_) => (),
                TilesetType::ImageAtlas { image, .. } => {
                    self.tileset_indexing.insert(*k, TilesetIndexing::Continious);
                    *v = TilesetType::Ready(textures.get_handle(image.to_owned()))
                },
                TilesetType::ImageCollection { collection, tile_size } => {
                    let mut map = HashMap::new();
                    let mut builder = TileAtlasBuilder::new(*tile_size);

                    for (tile_id, path) in collection {
                        let handle = textures.get_handle(path.to_owned());
                        map.insert(
                            *tile_id, 
                            builder.add_texture(
                                handle.clone(),
                                textures.get(&handle).expect("Image should be loaded"),
                            ).unwrap() as u16
                        );
                    }
                    self.tileset_indexing.insert(*k, TilesetIndexing::Special(map));
    
                    let atlas = builder.finish(textures).unwrap();

                    *v = TilesetType::Ready(atlas.texture)
                },
            }
        }
    } 
}

#[derive(Clone, Copy, Default)]
pub struct LevelLoader;

impl AssetLoader for LevelLoader {
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
            let fix_asset_path = |x: &PathBuf| AssetPath::new(x.strip_prefix(&root).unwrap().to_path_buf(), None);

            let mut dependencies = Vec::new();
            let mut handles = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                let tile_size = Vec2::new(tileset.tile_width as f32, tileset.tile_height as f32);

                match tileset.image.as_ref() {
                    Some(image) => {
                        warn!("Atlased tileset is not recommended. Please split your tileset into many tile files.");
                        let asset_path = fix_asset_path(&image.source);
                        
                        dependencies.push(asset_path.clone());
                        handles.insert(
                            tileset_index, 
                            TilesetType::ImageAtlas {
                                tile_size,
                                image: asset_path,
                            }
                        );
                    },
                    None => {
                        let collection = tileset.tiles()
                            .filter_map(|(tile_id, tile)| match &tile.image {
                                None => None,
                                Some(image) => {
                                    let asset_path = fix_asset_path(&image.source);
                                    dependencies.push(asset_path.clone());
                                    Some((tile_id, asset_path))
                                },
                            })
                            .collect();

                        handles.insert(
                            tileset_index, 
                            TilesetType::ImageCollection {
                                tile_size,
                                collection,
                            }
                        );
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

fn asset_dir_root() -> PathBuf {
    #[cfg(target_arch = "x86_64")]
    return bevy::asset::FileAssetIo::get_root_path();

    #[cfg(target_arch = "wasm32")]
    return PathBuf::new();
}
