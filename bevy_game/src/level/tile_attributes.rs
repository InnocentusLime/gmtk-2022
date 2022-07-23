use std::collections::HashMap;

use bevy::prelude::*;

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
    pub on_transition_graphics: Option<u32>,
    pub off_transition_graphics: Option<u32>,
    pub on_graphics: Option<u32>,
    pub off_graphics: Option<u32>,
    pub anim_type: Option<AnimationType>,
}

pub fn scan_properties(id: u32, tile: tiled::Tile) -> TileAttributes {
    use std::collections::HashSet;

    lazy_static::lazy_static! {
        static ref ATTR_TABLE: HashSet<String> = {
            let mut set = HashSet::new();
            set.insert("on_transition".to_owned());
            set.insert("off_transition".to_owned());
            set.insert("anim_type".to_owned());
            set.insert("active".to_owned());
            set.insert("on_tile".to_owned());
            set.insert("off_tile".to_owned());
            set.insert("anim_speed".to_owned());
            set
        };
    }

    let span = error_span!("Scanning tile properties", tile_id = id);
    let _guard = span.enter();

    let mut anim_type = None;
    let mut activation_cond = None;
    let mut on_graphics = None;
    let mut off_graphics = None;
    let mut anim_speed = None;
    let mut on_transition_graphics = None;
    let mut off_transition_graphics = None;

    for (name, prop) in tile.properties.iter() {
        if !ATTR_TABLE.contains(name.as_str()) {
            error!("Unknown property {}", name);
            continue;
        }
        match (name.as_str(), prop) {
            ("on_transition", tiled::PropertyValue::IntValue(i)) => {
                if *i < 0 { error!("Negative tile IDs are not allowed"); continue; }

                if on_transition_graphics.replace(*i as u32).is_some() {
                    warn!("On transition has been double-defined");
                }
            },
            ("off_transition", tiled::PropertyValue::IntValue(i)) => {
                if *i < 0 { error!("Negative tile IDs are not allowed"); continue; }

                if off_transition_graphics.replace(*i as u32).is_some() {
                    warn!("Off transition has been double-defined");
                }
            },
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
            ("on_tile", tiled::PropertyValue::IntValue(i)) => {
                if *i < 0 { error!("Negative tile IDs are not allowed"); continue; }

                if on_graphics.replace(*i as u32).is_some() {
                    warn!("On graphics have been double-defined");
                }
            },
            ("off_tile", tiled::PropertyValue::IntValue(i)) => {
                if *i < 0 { error!("Negative tile IDs are not allowed"); continue; }

                if off_graphics.replace(*i as u32).is_some() {
                    warn!("Off graphics have been double-defined");
                }
            },
            ("anim_speed", tiled::PropertyValue::FloatValue(f)) => {
                if *f < 0.0f32 { error!("Negative speed is not allowed"); continue; }

                if anim_speed.replace(*f).is_some() {
                    warn!("Off graphics have been double-defined");
                }
            },
            (x, _) => error!("Invalid data type for \"{}\" property", x),
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
            on_transition_graphics,
            off_transition_graphics,
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
