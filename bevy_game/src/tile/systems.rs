use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap_cpu_anim::{ CPUAnimated, CPUTileAnimations };
use crate::moveable::{ TileInteractionEvent, Moveable, MoveDirection, DecomposedRotation };
use crate::player::{ PlayerEscapedEvent, PlayerTag, PlayerWinnerTag };
use std::time::Duration;
use super::{ ActivationCondition, Active, ActivatableAnimating, FrierTag, ConveyorTag, EndTileTag };

pub struct LastPlayerSide(u8);

impl Default for LastPlayerSide {
    fn default() -> Self { Self(DecomposedRotation::new().upper_side()) }
}

pub fn tile_transition_animating(
    animations: Res<CPUTileAnimations>,
    player_q: Query<&Moveable, With<PlayerTag>>,
    mut tile_q: Query<(&ActivationCondition, &Active, &mut CPUAnimated, &ActivatableAnimating)>,
) {
    // Check if player has just started moving
    let try_change = player_q.get_single().map(Moveable::just_started_moving).unwrap_or(false);
    if !try_change { return; }
    
    let next_side = match player_q.get_single().ok().and_then(Moveable::next_side) {
        Some(x) => x,
        None => return,
    };
    tile_q.for_each_mut(|(cond, active, mut animated, animating)| {
        // Do not play transition animation for tiles that aren't going to 
        // change their state.
        if active.is_active == cond.is_active(next_side) { return; }

        match animating {
            ActivatableAnimating::Switch { on_transition, off_transition, .. } => animated.set_animation(
                if active.is_active { *off_transition } else { *on_transition },
                false,
                false,
                &*animations,
            ),
            _ => (),
        }
    });
}

// gjs
pub fn tile_animating_switch(
    animations: Res<CPUTileAnimations>,
    mut tile_q: Query<(&Active, &mut CPUAnimated, &ActivatableAnimating), Changed<Active>>,
) {
    tile_q.for_each_mut(|(active, mut animated, animating)| match animating {
        ActivatableAnimating::Switch { on_anim, off_anim, .. } => animated.set_animation(
            if active.is_active { *on_anim } else { *off_anim },
            false,
            true,
            &*animations,
        ),
        ActivatableAnimating::Pause { .. } => animated.paused = !active.is_active,
    });
}

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

                moveable.slide(dir, Duration::from_millis(500));
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
