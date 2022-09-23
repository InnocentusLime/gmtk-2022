use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::{ CPUAnimated, CPUTileAnimations };
use crate::moveable::{ TileInteractionEvent, MoveableQuery, Side as MoveableSide, MoveableBundle };
use crate::player::{ PlayerEscapedEvent, PlayerTag, PlayerWinnerTag };
use std::time::Duration;
use super::{ ActivationCondition, ActivatableAnimating, TileState, TileQuery, TileKind };

/// Switches tile animation if its state changes. See [ActivatableAnimating] for more info.
pub fn tile_animation_switch(
    animations: Res<CPUTileAnimations>,
    mut tile_q: Query<(&TileState, &mut CPUAnimated, &ActivatableAnimating), Changed<TileState>>,
) {
    tile_q.for_each_mut(|(state, mut animated, animating)| match state {
        TileState::Ready(x) => match animating {
            ActivatableAnimating::Switch { on_anim, off_anim, .. } => animated.set_animation(
                if *x { *on_anim } else { *off_anim }, 
                false, 
                true, 
                &*animations,
            ),
            ActivatableAnimating::Pause { .. } => animated.paused = !x,
        },
        TileState::Changing { to } => match animating {
            ActivatableAnimating::Switch { on_transition, off_transition, .. } => animated.set_animation(
                if *to { *on_transition } else { *off_transition }, 
                false, 
                false, 
                &*animations,
            ),
            ActivatableAnimating::Pause { .. } => (),
        },
    });
}

/// Switches tile state, depending on which number the player has on their upper side.
/// This system makes sure to trigger `Changed` only when the state of a tile actually changes.
pub fn tile_state_switching(
    player_q: Query<&MoveableSide, (With<PlayerTag>, Changed<MoveableSide>)>,
    mut tile_q: Query<(&mut TileState, &ActivationCondition)>, 
) {
    player_q.for_each(|side| match side {
        MoveableSide::Ready(x) => tile_q.for_each_mut(|(mut state, cond)| {
            let new_state = TileState::Ready(cond.is_active(*x));

            // Do the state change very carefully. We want to trigger
            // others only when we actually need to change the state.
            if new_state != *state {
                *state = new_state;
            }
        }),
        MoveableSide::Changing { from, to } => tile_q.for_each_mut(|(mut state, cond)| {
            let new_state = cond.is_active(*to);

            if cond.is_active(*from) != new_state {
                *state = TileState::Changing { to: new_state };
            }
        }),
    });
}

pub fn special_tile_handler(
    mut interactions: EventReader<TileInteractionEvent>,
    mut escape_event: EventWriter<PlayerEscapedEvent>,
    mut tile_query: Query<TileQuery>,
    mut move_query: Query<(MoveableQuery, Option<&PlayerTag>)>,
    mut commands: Commands,
) {
    let res: Result<(), anyhow::Error> = interactions.iter()
        .map(|e| {
            let tile = tile_query.get_mut(e.tile_id)?;
            let (moveable, player_tag) = move_query.get_mut(e.moveable_id)?;

            let (tile, mut moveable, is_player, moveable_id, _tile_id) = (
                tile, 
                moveable, 
                player_tag.is_some(), 
                e.moveable_id, 
                e.tile_id
            );

            if !tile.is_active() { return Ok(()); }

            match &tile.kind {
                // Conveyor logic
                TileKind::Conveyor => moveable.slide(tile.direction(), Duration::from_millis(500)),
                // Frier logic
                TileKind::FrierTile => commands.entity(moveable_id).despawn(),
                // Spinner logic
                TileKind::SpinningTile => moveable.rotate(tile.clock_wise(), Duration::from_millis(500)),
                // Exit logic
                TileKind::ExitTile if is_player => {
                    commands.entity(moveable_id)
                        .remove_bundle::<MoveableBundle>()
                        .insert(PlayerWinnerTag::new());
                    escape_event.send(PlayerEscapedEvent);
                },
                _ => (),
            };

            Ok(())
        })
        .collect();

    if let Err(e) = res {
        error!("Error while handling tile interaction: {}", e);
    }
}