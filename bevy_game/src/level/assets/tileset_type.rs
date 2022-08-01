use std::collections::HashMap;

use bevy::prelude::*;
use bevy::asset::AssetPath;

pub enum TilesetState {
    /// The tileset is already loaded and ready to be used.
    Ready {
        /// The handle to the tileset image.
        handle: Handle<Image>,
        /// The path to the tileset if it was an atlas. Image-collection based
        /// tilesets are constructed from scratch during the run, so their origin
        /// is a set of images.
        origin: Vec<AssetPath<'static>>,
    },
    /// The tileset isn't loaded yet. It's one atlas image.
    ImageAtlas {
        tile_size: Vec2,
        image: AssetPath<'static>,
    },
    /// The tileset isn't loaded yet. It's a collection of images
    /// with mapping from tile IDs to paths.
    ImageCollection {
        tile_size: Vec2,
        collection: HashMap<u32, AssetPath<'static>>,
    },
}

impl TilesetState {
    pub fn ready_image(&self) -> Handle<Image> {
        match self {
            Self::Ready { handle, .. } => handle.clone(),
            _ => panic!("The tileset isn't in a `ready` state. Make sure all assets have been loaded at this point"),
        }
    }
}
