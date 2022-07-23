use std::collections::HashMap;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use super::cpu_tile_animation::{ Animation, Animations };
use super::special_tiles::*;
use super::tile_def::*;
use super::tile_attributes::TileAttributes;
use super::tile_animation::dispatch_anim_type;

fn build_command(
    attr: &TileAttributes,
    anims: &HashMap<tiled::TileId, (f32, Animation)>,
    anim_ids: &mut HashMap<tiled::TileId, usize>,
    animations: &mut Animations,
) -> Box<dyn Fn(EntityCommands)> {
    type CommandPart = Box<dyn for<'a, 'b, 'c, 'd> Fn(&'d mut EntityCommands<'a, 'b, 'c>) -> &'d mut EntityCommands<'a, 'b, 'c> + 'static>;

    let tagger: CommandPart = match attr.tile_ty.as_ref().map(String::as_str) {
        Some("player_start") => Box::new(|x| { x.insert(StartTileTag) }),
        Some("player_end") => Box::new(|x| { x.insert(EndTileTag) }),
        Some("conveyor") => Box::new(|x| { x.insert(ConveyorTag) }),    
        Some("fry") => Box::new(|x| { x.insert(FrierTag) }),    
        Some(x) => { error!("Unknown tile type \"{}\"", x); Box::new(|x| x) },
        None => Box::new(|x| { x.insert(SolidTileTag) }),
    };

    let active_tagger: CommandPart = match (attr.activation_cond, attr.anim_type) {
        (None, None) => Box::new(|x| x),
        (Some(_), None) | (None, Some(_)) => { error!("Incomplete activatable tile info"); Box::new(|x| x) },
        (Some(cond), Some(anim_type)) => match dispatch_anim_type(anim_type, attr, anims, anim_ids, animations) {
            Some((act_anim_ty, cpu_animated)) => Box::new(move |x| { 
                x.insert(ActivatableTileTag::new(cond, act_anim_ty))
                    .insert(cpu_animated) 
            }),
            None => Box::new(|x| x), 
        },
    };
                        
    Box::new(move |mut x| { active_tagger(tagger(&mut x)); })
}

/// Parsees tile attributes, converting them into commands, which map
/// tile ID into tile initialization routine
pub fn build_commands(
    attrs: &Vec<HashMap<tiled::TileId, TileAttributes>>,
    anims: &Vec<HashMap<tiled::TileId, (f32, Animation)>>,
    anim_ids: &mut Vec<HashMap<tiled::TileId, usize>>,
    animations: &mut Animations,
) -> Vec<HashMap<tiled::TileId, Box<dyn Fn(EntityCommands)>>> {
    let span = error_span!("Building commands");
    let _guard = span.enter();
    attrs.iter().enumerate().map(|(tileset_id, attrs)| {
        let span = error_span!("Bulding commands for tileset", tileset_id = tileset_id);
        let _guard = span.enter();
        attrs.iter().map(|(id, attrs)| {
            let span = error_span!("Building command for tile", tile_id = id);
            let _guard = span.enter();
            (*id, build_command(attrs, &anims[tileset_id], &mut anim_ids[tileset_id], animations))
        }).collect()
    }).collect()
}
    
