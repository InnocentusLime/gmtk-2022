use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;

use super::tile_attributes::{ TileAttributes, AnimationType };
use super::cpu_tile_animation::{ Frame, Animation, Animations, CPUAnimated };
use super::tile_def::*;

// TODO be able to suggest looping
/// Gets tile animation with a suggested speed
/// multiplier
fn get_tile_animation(
    id: u32, 
    tile: tiled::Tile,
    attr: &TileAttributes,
) -> (f32, Animation) {
    let anim = match tile.animation.as_ref() {
        None => Animation::from_frames(std::iter::once(Frame { 
            texture_id: id as u16,
            duration: Duration::new(1, 1),
        })),
        Some(x) => Animation::from_frames(x.iter().map(|frame| Frame {
            texture_id: frame.tile_id as u16,
            duration: Duration::from_millis(frame.duration as u64),
        })),
    };

    (attr.anim_speed.unwrap_or(1.0f32), anim)
}

/// Scans the tilesets, looking for anims
pub fn scan_tilesets_for_animations(
    map: &tiled::Map, 
    attributes: &Vec<HashMap<tiled::TileId, TileAttributes>>,
) -> Vec<HashMap<tiled::TileId, (f32, Animation)>> {
    map.tilesets().iter().enumerate().map(|(tileset_id, tileset)| { 
        let span = error_span!("Scanning tileset", tileset_id = tileset_id);
        let _guard = span.enter();
        tileset.tiles().map(|(id, tile)| (id, get_tile_animation(id, tile, &attributes[tileset_id][&id])))
            .collect()
    }).collect()
}

pub fn dispatch_anim_type(
    ty: AnimationType,
    attr: &TileAttributes,
    anims: &HashMap<tiled::TileId, (f32, Animation)>,
    anim_ids: &mut HashMap<tiled::TileId, usize>,
    animations: &mut Animations,
) -> Option<(ActivatableAnimating, CPUAnimated)> {
    match (ty, attr.on_graphics, attr.off_graphics) {
        (AnimationType::Switch, Some(tile_on), Some(tile_off)) => {
            let (on_anim_speed, on_anim) = &anims[&tile_on];
            let (off_anim_speed, off_anim) = &anims[&tile_off];
            let on_anim_id = *anim_ids.entry(tile_on)
                .or_insert_with(|| animations.add_animation(on_anim.clone()));
            let off_anim_id = *anim_ids.entry(tile_off)
                .or_insert_with(|| animations.add_animation(off_anim.clone()));

            let on_anim = animations.new_cpu_animated(on_anim_id, *on_anim_speed, true);
            let off_anim = animations.new_cpu_animated(off_anim_id, *off_anim_speed, true);
            Some((ActivatableAnimating::Switch {
                on_anim, off_anim
            }, on_anim))
        },
        (AnimationType::Switch, _, _) => { error!("Missing some data for \"switchable_machine\" tile type"); None },
        (AnimationType::Stop, Some(tile_on), x) => {
            if x.is_some() { warn!("\"stoppable_machine\" animation type ignores \"tile_off\"") }
            let (on_anim_speed, on_anim) = &anims[&tile_on];
            let on_anim_id = *anim_ids.entry(tile_on)
                .or_insert_with(|| animations.add_animation(on_anim.clone()));

            let on_anim = animations.new_cpu_animated(on_anim_id, *on_anim_speed, true);
            Some((ActivatableAnimating::Stop {
                on_speed: *on_anim_speed,
            }, on_anim))
        },
        (AnimationType::Stop, _, _) => { error!("Missing some data for \"stoppable_machine\" tile type"); None },
    }
} 
