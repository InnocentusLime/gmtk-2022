use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;
use crate::moveable::{ TileInteractionEvent, Moveable, MoveDirection };
use crate::player::{ PlayerMoved, PlayerChangingSide, PlayerEscapedEvent, PlayerTag, PlayerWinnerTag };
use std::time::Duration;
use super::{ ActivatableTileTag, ActivatableAnimating, FrierTag, ConveyorTag, EndTileTag };

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

pub fn fry_logic(
    mut interactions: EventReader<TileInteractionEvent>,
    mut tile_query: Query<&ActivatableTileTag, With<FrierTag>>,
    mut move_query: Query<(), With<Moveable>>,
    mut commands: Commands,
) {
    for e in interactions.iter() {
        match (tile_query.get_mut(e.tile_id), move_query.get_mut(e.interactor_id)) {
            (Ok(state), Ok(_)) if state.is_active() => commands.entity(e.interactor_id).despawn(),
            _ => (),
        }
    }
}

pub fn conveyor_logic(
    mut interactions: EventReader<TileInteractionEvent>,
    mut tile_query: Query<(&ActivatableTileTag, &Tile), With<ConveyorTag>>,
    mut move_query: Query<&mut Moveable>,
) {
    for e in interactions.iter() {
        match (tile_query.get_mut(e.tile_id), move_query.get_mut(e.interactor_id)) {
            (Ok((state, tile)), Ok(mut moveable)) if state.is_active() => {
                let dir = MoveDirection::Up.apply_flipping_flags(tile.flip_x, tile.flip_y, tile.flip_d);
                moveable.slide(dir, Duration::from_millis(132));
            },
            _ => (),
        }
    }
}

pub fn exit_logic(
    mut interactions: EventReader<TileInteractionEvent>,
    mut tile_query: Query<(), With<EndTileTag>>,
    mut move_query: Query<(), (With<PlayerTag>, With<Moveable>)>,
    mut escape_event: EventWriter<PlayerEscapedEvent>,
    mut commands: Commands,
) {
    for e in interactions.iter() {
        match (tile_query.get_mut(e.tile_id), move_query.get_mut(e.interactor_id)) {
            (Ok(_), Ok(_)) => {
                commands.entity(e.interactor_id)
                    .remove::<Moveable>()
                    .insert(PlayerWinnerTag::new());
                escape_event.send(PlayerEscapedEvent);
            },
            _ => (),
        }
    }
}
