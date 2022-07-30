use std::collections::HashMap;

use bevy::prelude::*;
use bevy::asset::AssetPath;

pub enum TilesetType {
    Ready(Handle<Image>),
    ImageAtlas {
        tile_size: Vec2,
        image: AssetPath<'static>,
    },
    ImageCollection {
        tile_size: Vec2,
        collection: HashMap<u32, AssetPath<'static>>,
    },
}

impl TilesetType {
    pub fn ready_image(&self) -> Handle<Image> {
        match self {
            Self::Ready(image) => image.clone(),
            _ => panic!("The image wasn't in a `ready` state. Make sure all assets have been initialized at this point"),
        }
    }
}
