use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;

use crate::player::{ PlayerMoved, PlayerModification, PlayerChangingSide };

use super::{ ActivatableTileTag, ActivatableAnimating, TileState };

pub(super) fn toggle_activatable_tiles<F>(
    mut filter: F,
    mut query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>, 
) 
where
    F: FnMut(&mut ActivatableTileTag) -> bool
{
    for (mut cpu_anim, mut active_tag) in query.iter_mut() {
        if filter(&mut *active_tag) {
            active_tag.anim_info.update_cpu_anim(&mut *cpu_anim, active_tag.is_active());
        }
    }
}

pub fn activeatable_tile_transition_system(
    mut change_side: EventReader<PlayerChangingSide>,
    mut query: Query<(&mut CPUAnimated, &ActivatableTileTag)>
) {
    if let Some(e) = change_side.iter().next() {
        for (mut anim, tag) in query.iter_mut() {
            if tag.is_active() == tag.will_be_active(&e.dice_state) { continue; }
            match tag.anim_info {
                ActivatableAnimating::Switch { on_transition, off_transition, .. } => {
                    if tag.is_active() {
                        *anim = off_transition;
                    } else {
                        *anim = on_transition;
                    }
                },
                _ => (),
            }
        }
    }

    // Drop all other events
    change_side.iter().for_each(|_| ());
}

pub fn tile_reaction_system<T: TileState>(
    mut moves: EventReader<PlayerMoved>,
    mut query: Query<(&mut T, T::UpdateData)>,
    mut response: EventWriter<PlayerModification>,
) {
    use std::any::type_name;
    for e in moves.iter() {
        if let Ok((mut state, extra)) = query.get_mut(e.cell) {
            trace!("{} reacted to player", type_name::<T>());
            response.send(state.react(extra));
        }
    }
}

pub fn tile_switch_system(
    mut moves: EventReader<PlayerMoved>,
    query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>, 
) {
    if let Some(e) = moves.iter().next() {
        toggle_activatable_tiles(|tag| tag.update(&e.dice_state), query);
    }

    // Drop all other events
    moves.iter().for_each(|_| ());
}

