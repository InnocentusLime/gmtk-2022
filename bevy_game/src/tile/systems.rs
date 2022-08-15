use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;
use crate::moveable::{ TileInteractionEvent, Moveable, MoveDirection, DecomposedRotation };
use crate::player::{ PlayerEscapedEvent, PlayerTag, PlayerWinnerTag };
use std::time::Duration;
use super::{ ActivationCondition, Active, ActivatableAnimating, FrierTag, ConveyorTag, EndTileTag };

pub struct LastPlayerSide(u8);

impl Default for LastPlayerSide {
    fn default() -> Self { Self(DecomposedRotation::new().upper_side()) }
}

// TODO implement via "Changed"
/*
pub fn tile_transition_animating(
    mut last_next_side: Local<LastPlayerSide>,
    player_q: Query<&Moveable, With<PlayerTag>>,
    mut tile_q: Query<(&mut CPUAnimated, &ActivatableTileTag)>,
) {
    player_q.for_each(|moveable| {
        if let Some(next_side) = moveable.next_side() {
            if last_next_side.0 != next_side {
                last_next_side.0 = next_side;

                tile_q.for_each_mut(|(mut anim, tag)| {
                    if tag.is_active() == tag.will_be_active(next_side) { return; }
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
                })
            }
        }
    });
}
*/

pub fn tile_state_switching(
    player_q: Query<&Moveable, With<PlayerTag>>,
    mut tile_q: Query<(&mut Active, &ActivationCondition)>, 
) {
    player_q.for_each(|moveable| 
        tile_q.for_each_mut(|(mut active, cond)| {
            let new_state = cond.is_active(moveable.upper_side());

            if new_state != active.is_active {
                // This will trigger a change only when it's neccesary
                active.is_active = new_state;
            }
        })
    );
}

pub fn frier_tile_handler(
    mut interactions: EventReader<TileInteractionEvent>,
    mut tile_query: Query<&Active, With<FrierTag>>,
    mut move_query: Query<(), With<Moveable>>,
    mut commands: Commands,
) {
    for e in interactions.iter() {
        match (tile_query.get_mut(e.tile_id), move_query.get_mut(e.interactor_id)) {
            (Ok(state), Ok(_)) if state.is_active => commands.entity(e.interactor_id).despawn(),
            _ => (),
        }
    }
}

pub fn conveyor_tile_handler(
    mut interactions: EventReader<TileInteractionEvent>,
    mut tile_query: Query<(&Active, &TileFlip), With<ConveyorTag>>,
    mut move_query: Query<&mut Moveable>,
) {
    for e in interactions.iter() {
        match (tile_query.get_mut(e.tile_id), move_query.get_mut(e.interactor_id)) {
            (Ok((state, flip)), Ok(mut moveable)) if state.is_active => {
                let dir = MoveDirection::Up.apply_flipping_flags(flip.x, flip.y, flip.d);

                moveable.slide(dir, Duration::from_millis(132));
            },
            _ => (),
        }
    }
}

pub fn exit_tile_handler(
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
