//! Module which houses `tiled` TileMap conviniently wrapped into an asset
//! together with its own asset loader.

use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;

use bevy::asset::{ AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset };
use bevy_ecs_tilemap::prelude::TilemapTexture;
use bevy_ecs_tilemap_cpu_anim::{ Frame, CPUTileAnimation };
use bevy::prelude::*;
use bevy::reflect::TypeUuid;

pub fn tileset_indexing(
    In(map): In<Handle<TiledMap>>,
    images: Res<Assets<Image>>,
    maps: Res<Assets<TiledMap>>,
) -> Vec<(TilesetIndexing, TilemapTexture)> {
    let map = maps.get(&map).unwrap();
    map.tilesets.iter()
        .map(|(_, tileset)| match tileset {
            TiledTileset::Image(path) => (
                TilesetIndexing::Continious,
                TilemapTexture::Single(images.get_handle(path.to_owned()))
            ),
            TiledTileset::ImageCollection(tiles) => (
                TilesetIndexing::Special(
                    tiles.iter()
                    .map(|(id, _)| *id)
                    .enumerate()
                    .map(|(to, from)| (from, to as u32))
                    .collect()
                ),
                TilemapTexture::Vector(
                    tiles.iter()
                    .map(|(_, path)| images.get_handle(path.to_owned()))
                    .collect()
                )
            ),
        })
        .collect()
}

/// A type, which encodes mapping from `Tiled` tile IDs to
/// engine's IDs in the tile atlas.
#[derive(Debug)]
pub enum TilesetIndexing {
    Continious,
    Special(HashMap<u32, u32>),
}

impl TilesetIndexing {
    /// Constructs the mapping, given the compiled atlas
    /// and the tileset source info.
    pub fn from_tileset_and_atlas(
        tileset_info: &TiledTileset,
        atlas: &TextureAtlas,
    ) -> Self {
        match tileset_info {
            TiledTileset::Image(_) => Self::Continious,
            TiledTileset::ImageCollection(c) => Self::Special(
                c.iter().map(|(from, path)| (
                        *from,
                        *atlas.texture_handles.as_ref().and_then(|map|
                            map.get(&Handle::weak(path.get_id().into()))
                        )
                        .unwrap() as u32
                    )
                ).collect()
            ),
        }
    }

    /// Maps the tile ID from `Tiled` to the engine's tile ID.
    pub fn dispatch(&self, x: u32) -> u32 {
        match self {
            TilesetIndexing::Continious => x,
            TilesetIndexing::Special(map) => map[&x],
        }
    }

    /// Maps a tiled animation into an CPUTileAnimation
    pub fn cpu_tile_anim(&self, anim: &[tiled::Frame]) -> CPUTileAnimation {
        CPUTileAnimation::new(anim.iter().map(|frame| Frame {
            texture_id: self.dispatch(frame.tile_id),
            duration: Duration::from_millis(frame.duration as u64),
        }))
    }
}

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
            let mut dependencies = Vec::new();

            for tileset in map.tilesets() {
                let tile_size = Vec2::new(tileset.tile_width as f32, tileset.tile_height as f32);
                let tileset = match tileset.image.as_ref() {
                    Some(image) => {
                        let asset_path = fix_asset_path(&image.source);
                        dependencies.push(asset_path.clone());

                        TiledTileset::Image(asset_path)
                    },
                    None => {
                        let asset_paths: Vec<(u32, AssetPath<'static>)> = tileset.tiles()
                            .filter_map(|(tile_id, tile)|
                                tile.image.as_ref().map(|x| (tile_id, fix_asset_path(&x.source)))
                            )
                            .collect();
                        asset_paths.iter().for_each(|(_, path)| dependencies.push(path.to_owned()));

                        TiledTileset::ImageCollection(asset_paths)
                    },
                };

                tilesets.push((tile_size, tileset));
            }

            let loaded_asset = LoadedAsset::new(TiledMap {
                map, tilesets,
            }).with_dependencies(dependencies);

            load_context.set_default_asset(loaded_asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] { &["tmx"] }
}
