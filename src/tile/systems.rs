use anyhow::anyhow;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileStorage, TilePos};
use bevy_ecs_tilemap_cpu_anim::{ CPUAnimated, CPUTileAnimations };
use crate::moveable::{ TileInteractionEvent, MoveableQuery, Side as MoveableSide, MoveableBundle };
use crate::player::{ PlayerEscapedEvent, PlayerTag, PlayerWinnerTag };
use std::time::Duration;
use super::{ ActivationCondition, ActivatableAnimating, TileState, LogicTileQuery, TileKind, LogicTilemapTag, GraphicsTilemapTag };

/// Switches tile animation if its state changes. See [ActivatableAnimating] for more info.
pub fn tile_animation_switch(
    animations: Res<CPUTileAnimations>,
    graphics_map_q: Query<&TileStorage, With<GraphicsTilemapTag>>,
    logic_q: Query<(&TileState, &TilePos), Changed<TileState>>,
    mut graphics_q: Query<(&mut CPUAnimated, &ActivatableAnimating)>,
) {
    logic_q.for_each(|(state, pos)| graphics_map_q.for_each(|storage| {
        let entity = match storage.get(pos) {
            Some(x) => x,
            None => return,
        };
        let (mut animated, animating) = match graphics_q.get_mut(entity) {
            Ok(x) => x,
            Err(_) => return,
        };

        match state {
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
        }
    }));
}

/// Switches tile state, depending on which number the player has on their upper side.
/// This system makes sure to trigger `Changed` only when the state of a tile actually changes.
pub fn tile_state_switching(
    player_q: Query<&MoveableSide, (With<PlayerTag>, Changed<MoveableSide>)>,
    trigger_q: Query<(&ActivationCondition, &TilePos)>, 
    logic_map_q: Query<&TileStorage, With<LogicTilemapTag>>,
    mut logic_q: Query<&mut TileState>, 
) {
    let player_side = match player_q.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let logic_map = match logic_map_q.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let log_error = |x: Result<(), anyhow::Error>| match x {
        Ok(()) => (),
        Err(e) => { error!("{}", e); }
    };

    match player_side {
        MoveableSide::Ready(x) => trigger_q.iter().map(|(cond, pos)| {
            let mut state = logic_q.get_mut(
                logic_map.get(pos)
                    .ok_or(anyhow!("Trigger at {:?} has no target", pos))?
            )?;
            let new_state = TileState::Ready(cond.is_active(*x));

            // Do the state change very carefully. We want to trigger
            // others only when we actually need to change the state.
            if new_state != *state {
                *state = new_state;
            }

            Ok(())
        })
        .for_each(log_error),
        MoveableSide::Changing { from, to } => trigger_q.iter().map(|(cond, pos)| {
            let mut state = logic_q.get_mut(
                logic_map.get(pos)
                    .ok_or(anyhow!("Trigger at {:?} has no target", pos))?
            )?;
            let new_state = cond.is_active(*to);

            if cond.is_active(*from) != new_state {
                *state = TileState::Changing { to: new_state };
            }

            Ok(())
        })
        .for_each(log_error),
    }
}

pub fn special_tile_handler(
    mut interactions: EventReader<TileInteractionEvent>,
    mut escape_event: EventWriter<PlayerEscapedEvent>,
    mut tile_query: Query<LogicTileQuery>,
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
                TileKind::Frier => commands.entity(moveable_id).despawn(),
                // Spinner logic
                TileKind::Spinner => moveable.rotate(tile.clock_wise(), Duration::from_millis(500)),
                // Exit logic
                TileKind::Exit if is_player => {
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