use std::collections::HashMap;

use bevy::prelude::*;

use super::special_tiles::*;
use super::tile_def::*;

#[derive(Clone, Copy, Debug)]
pub enum AnimationType {
    Switch,
    Stop,
}

/// A relatively raw set of attributes about a tile
/// that the game can recognize
#[derive(Clone)]
pub struct TileAttributes {
    pub tile_ty: Option<String>,
    pub activation_cond: Option<ActivationCondition>,
    pub anim_speed: Option<f32>,
    pub on_graphics: Option<u32>,
    pub off_graphics: Option<u32>,
    pub anim_type: Option<AnimationType>,
}

pub fn scan_properties(id: u32, tile: tiled::Tile) -> TileAttributes {
    let span = error_span!("Scanning tile properties", tile_id = id);
    let _guard = span.enter();

    let mut anim_type = None;
    let mut activation_cond = None;
    let mut on_graphics = None;
    let mut off_graphics = None;
    let mut anim_speed = None;
    for (name, prop) in tile.properties.iter() {
        match (name.as_str(), prop) {
            ("anim_type", tiled::PropertyValue::StringValue(s)) => {
                let x = match s.as_str() {
                    "switch" => AnimationType::Switch,
                    "stop" => AnimationType::Stop,
                    x => { error!("Invalid animation type: {}", x); continue },
                };

                if anim_type.replace(x).is_some() {
                    warn!("Animation type has been double-defined");
                }
            },
            ("anim_type", _) => error!("Invalid data type for \"anim_type\" property"),
            ("active", tiled::PropertyValue::StringValue(s)) => {
                let x = match s.as_str() {
                    "odd" => ActivationCondition::Odd,
                    "even" => ActivationCondition::Even,
                    x => { error!("Invalid activation value: {}", x); continue },
                };

                if activation_cond.replace(x).is_some() {
                    warn!("Activation condition has been double-defined");
                }
            },
            ("active", _) => error!("Invalid data type for \"active\" property"),
            ("on_tile", tiled::PropertyValue::IntValue(i)) => {
                if *i < 0 { error!("Negative tile IDs are not allowed"); continue; }

                if on_graphics.replace(*i as u32).is_some() {
                    warn!("On graphics have been double-defined");
                }
            },
            ("on_tile", _) => error!("Invalid data type for \"on_tile\" property"),
            ("off_tile", tiled::PropertyValue::IntValue(i)) => {
                if *i < 0 { error!("Negative tile IDs are not allowed"); continue; }

                if off_graphics.replace(*i as u32).is_some() {
                    warn!("Off graphics have been double-defined");
                }
            },
            ("off_tile", _) => error!("Invalid data type for \"off_tile\" property"),
            ("anim_speed", tiled::PropertyValue::FloatValue(f)) => {
                if *f < 0.0f32 { error!("Negative speed is not allowed"); continue; }

                if anim_speed.replace(*f).is_some() {
                    warn!("Off graphics have been double-defined");
                }
            },
            ("anim_speed", _) => error!("Invalid data type for \"anim_speed\" property"),
            (x, _) => error!("Unknown property {}", x),
        }
    }

    let res =  
        TileAttributes {
            tile_ty: tile.tile_type.clone(),
            activation_cond,
            on_graphics,
            off_graphics,
            anim_type,
            anim_speed,
        };

    res
}

/// Scans the tilesets, looking for special tile defs
pub fn scan_tilesets(map: &tiled::Map) -> Vec<HashMap<tiled::TileId, TileAttributes>> {
    map.tilesets().iter().enumerate().map(|(tileset_id, tileset)| { 
        let span = error_span!("Scanning tileset", tileset_id = tileset_id);
        let _guard = span.enter();
        tileset.tiles().map(|(id, tile)| (id, scan_properties(id, tile))).collect()
    }).collect()
}
