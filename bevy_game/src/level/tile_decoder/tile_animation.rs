use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::{ Frame, CPUTileAnimation, CPUTileAnimations, CPUAnimated };

use crate::tile::ActivatableAnimating;
use super::tile_attributes::{ TileAttributes, AnimationType };

// TODO be able to suggest looping
/// Gets tile animation with a suggested speed
/// multiplier
fn get_tile_animation(
    id: u32, 
    tile: tiled::Tile,
    attr: &TileAttributes,
) -> (f32, CPUTileAnimation) {
    let anim = match tile.animation.as_ref() {
        None => CPUTileAnimation::from_frames(std::iter::once(Frame { 
            texture_id: id,
            duration: Duration::new(1, 1),
        })),
        Some(x) => CPUTileAnimation::from_frames(x.iter().map(|frame| Frame {
            texture_id: frame.tile_id,
            duration: Duration::from_millis(frame.duration as u64),
        })),
    };

    (attr.anim_speed.unwrap_or(1.0f32), anim)
}

/// Scans the tilesets, looking for anims
pub fn scan_tilesets_for_animations(
    map: &tiled::Map, 
    attributes: &Vec<HashMap<tiled::TileId, TileAttributes>>,
) -> Vec<HashMap<tiled::TileId, (f32, CPUTileAnimation)>> {
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
    anims: &HashMap<tiled::TileId, (f32, CPUTileAnimation)>,
    anim_ids: &mut HashMap<tiled::TileId, usize>,
    animations: &mut CPUTileAnimations,
) -> Option<(ActivatableAnimating, CPUAnimated)> {
    match (ty, attr.on_graphics, attr.off_graphics, attr.on_transition_graphics, attr.off_transition_graphics) {
        (AnimationType::Switch, Some(tile_on), Some(tile_off), Some(on_transition), Some(off_transition)) => {
            let (on_anim_speed, on_anim) = &anims[&tile_on];
            let (off_anim_speed, off_anim) = &anims[&tile_off];
            let (on_transition_speed, on_transition_anim) = &anims[&on_transition];
            let (off_transition_speed, off_transition_anim) = &anims[&off_transition];
            let on_anim_id = *anim_ids.entry(tile_on)
                .or_insert_with(|| animations.add_animation(on_anim.to_owned()));
            let off_anim_id = *anim_ids.entry(tile_off)
                .or_insert_with(|| animations.add_animation(off_anim.to_owned()));
            let on_transition_id = *anim_ids.entry(on_transition)
                .or_insert_with(|| animations.add_animation(on_transition_anim.to_owned()));
            let off_transition_id = *anim_ids.entry(off_transition)
                .or_insert_with(|| animations.add_animation(off_transition_anim.to_owned()));

            let on_anim = animations.new_cpu_animated(on_anim_id, *on_anim_speed, true);
            let off_anim = animations.new_cpu_animated(off_anim_id, *off_anim_speed, true);
            let on_transition = animations.new_cpu_animated(on_transition_id, *on_transition_speed, false);
            let off_transition = animations.new_cpu_animated(off_transition_id, *off_transition_speed, false);
            Some((ActivatableAnimating::Switch {
                on_anim, off_anim,
                on_transition, off_transition
            }, on_anim))
        },
        (AnimationType::Switch, _, _, _, _) => { error!("Missing some data for \"switchable_machine\" tile type"); None },
        (AnimationType::Stop, Some(tile_on), x1, x2, x3) => {
            if x1.is_some() { warn!("\"stop\" animation type ignores \"off_tile\"") }
            if x2.is_some() { warn!("\"stop\" animation type ignores \"on_transition\"") }
            if x3.is_some() { warn!("\"stop\" animation type ignores \"off_transition\"") }

            let (on_anim_speed, on_anim) = &anims[&tile_on];
            let on_anim_id = *anim_ids.entry(tile_on)
                .or_insert_with(|| animations.add_animation(on_anim.to_owned()));

            let on_anim = animations.new_cpu_animated(on_anim_id, *on_anim_speed, true);
            Some((ActivatableAnimating::Stop {
                on_speed: *on_anim_speed,
            }, on_anim))
        },
        (AnimationType::Stop, _, _, _, _) => { error!("Missing some data for \"stoppable_machine\" tile type"); None },
    }
} 
