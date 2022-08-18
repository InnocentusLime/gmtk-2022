//! Module which houses extension-traits to quickly get some tile data in
//! `bevy_ecs_tilemap` readable format. This module also houses simple wrappers
//! to read (deserialize) tile properties into more usable data in a non-painful
//! way.

mod deser_impl;

use bevy_ecs_tilemap::prelude::*;
use serde::Deserialize;

pub use deser_impl::*;

pub trait TiledLayerTileExt {
    fn bevy_flip_flags(&self) -> TileFlip;
}

impl TiledLayerTileExt for tiled::LayerTileData {
    fn bevy_flip_flags(&self) -> TileFlip {
        TileFlip {
            x: self.flip_h,
            y: self.flip_v,
            d: self.flip_d,
        }
    }
}

pub trait TiledTileExt<'de> {
    fn deser_properties<D: Deserialize<'de>>(&'de self) ->  Result<D, TilePropertyDeserError>;
}

impl<'de> TiledTileExt<'de> for tiled::Tile<'de> {
    fn deser_properties<D: Deserialize<'de>>(&'de self) -> Result<D, TilePropertyDeserError> {
        D::deserialize(TilePropertyDes { tile: self })
    }
}
