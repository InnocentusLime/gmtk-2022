//! Module which houses extension-traits to quickly get some tile data in
//! `bevy_ecs_tilemap` readable format. This module also houses simple wrappers
//! to read (deserialize) tile properties into more usable data in a non-painful
//! way.

mod deser_impl;

use bevy_ecs_tilemap::prelude::*;
use serde::{ Deserialize, de::DeserializeOwned, Deserializer };
use std::collections::HashMap;
use tiled::{ Layer, Map, LayerType, TileLayer, FiniteTileLayer, GroupLayer };

pub use deser_impl::*;

macro_rules! impl_find_layer {
    ($(fn $name:ident($ident:ident in $pat:pat_param) -> $result:ty { ... })+) => {
        $(fn $name(&self, name: &str) -> Option<$result> {
            self.layers().find(|x| x.name.as_str() == name)
                .and_then(|x| match x.layer_type() {
                    $pat => Some($ident),
                    _ => None,
                })
        })+
    };
}

pub trait TilesetExt {
    fn tile_properties<D: DeserializeOwned>(&self) -> Result<HashMap<tiled::TileId, D>, TilePropertyDeserError>;
}

impl TilesetExt for tiled::Tileset {
    fn tile_properties<D: DeserializeOwned>(&self) -> Result<HashMap<tiled::TileId, D>, TilePropertyDeserError> {
        self.tiles()
            .map(|(id, tile)| tile.properties().map(|prop| (id, prop)))
            .collect()
    }
    
    // `tiled` has screwed up lifetime nonsense
    /*
    fn tile_properties<D: Deserialize<'de>>(&'de self) -> Result<HashMap<tiled::TileId, D>, TilePropertyDeserError> {
        self.tiles()
            .map(|(id, tile)| tile.deref().properties().map(|prop| (id, prop)))
            .collect()
    }
    */
}

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

pub trait TileExt<'de> {
    fn properties<D: Deserialize<'de>>(&'de self) -> Result<D, TilePropertyDeserError>;
}

impl<'de> TileExt<'de> for tiled::Tile<'de> {
    fn properties<D: Deserialize<'de>>(&'de self) -> Result<D, TilePropertyDeserError> {
        D::deserialize(TilePropertyDes { tile: self })
    }
}

pub trait LayerSearch {
    fn layers(&self) -> Box<dyn ExactSizeIterator<Item = Layer<'_>> + '_>;

    impl_find_layer!(
        fn finite_tile_layer(layer in LayerType::TileLayer(TileLayer::Finite(layer))) -> FiniteTileLayer { ... }
        fn group_layer(layer in LayerType::GroupLayer(layer)) -> GroupLayer { ... }
    );
}

impl LayerSearch for Map {
    fn layers(&self) -> Box<dyn ExactSizeIterator<Item = Layer<'_>> + '_> { Box::new(self.layers()) }
}

impl<'a> LayerSearch for GroupLayer<'a> {
    fn layers(&self) -> Box<dyn ExactSizeIterator<Item = Layer<'_>> + '_> { Box::new(self.layers()) }
}

pub fn deserailize_from_json_str<'de, D, T>(des: D) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let s = <&'_ str as Deserialize>::deserialize(des)?;
    serde_json::from_str(s)
        .map_err(<D::Error as serde::de::Error>::custom)
}