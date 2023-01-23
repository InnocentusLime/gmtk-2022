use super::{
    ActivatableAnimating, ActivationCondition, GraphicsTilemapTag,
    LogicTileQuery, LogicTilemapTag, TileKind, TileState,
};
use crate::moveable::{
    MoveableBundle, MoveableQuery, Side as MoveableSide, TileInteractionEvent,
};
use crate::player::{PlayerEscapedEvent, PlayerTag, PlayerWinnerTag};
use anyhow::anyhow;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;
use std::ops::Deref;
use std::time::Duration;

pub fn tile_animation_setup(
    mut commands: Commands,
    tile_q: Query<
        (Entity, &ActivatableAnimating),
        Added<ActivatableAnimating>,
    >,
) {
    tile_q.for_each(|(entity, animating)| match animating {
        ActivatableAnimating::None => (),
        ActivatableAnimating::Switch { on_anim, .. }
        | ActivatableAnimating::Pause { on_anim } => {
            commands.entity(entity).insert(CPUAnimated::new(
                on_anim.clone(),
                true,
                false,
            ));
        }
    })
}

/// Switches tile animation if its state changes. See [ActivatableAnimating] for more info.
pub fn tile_animation_switch(
    graphics_map_q: Query<&TileStorage, With<GraphicsTilemapTag>>,
    logic_q: Query<(&TileState, &TilePos), Changed<TileState>>,
    mut graphics_q: Query<(&mut CPUAnimated, &ActivatableAnimating)>,
) {
    logic_q.for_each(|(state, pos)| {
        graphics_map_q.for_each(|storage| {
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
                    ActivatableAnimating::Switch {
                        on_anim, off_anim, ..
                    } => animated.set_animation(
                        if *x { on_anim } else { off_anim }.clone(),
                        false,
                        true,
                    ),
                    ActivatableAnimating::Pause { .. } => animated.paused = !x,
                    _ => (),
                },
                TileState::Changing { to } => match animating {
                    ActivatableAnimating::Switch {
                        on_transition,
                        off_transition,
                        ..
                    } => animated.set_animation(
                        if *to { on_transition } else { off_transition }
                            .clone(),
                        false,
                        false,
                    ),
                    ActivatableAnimating::Pause { .. } => (),
                    _ => (),
                },
            }
        })
    });
}

/// Switches tile state, depending on which number the player has on their upper side.
/// This system makes sure to trigger `Changed` only when the state of a tile actually changes.
pub fn tile_state_switching(
    player_q: Query<&MoveableSide, (With<PlayerTag>, Changed<MoveableSide>)>,
    trigger_q: Query<(&ActivationCondition, &TilePos)>,
    logic_map_q: Query<&TileStorage, With<LogicTilemapTag>>,
    logic_q: Query<&mut TileState>,
) {
    let player_side = match player_q.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let logic_map = match logic_map_q.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let res: anyhow::Result<()> = match player_side {
        MoveableSide::Ready(x) => apply_on_logic(
            logic_map, logic_q, trigger_q.iter(),
            |cond, mut state| {
                let new_state = TileState::Ready(cond.is_active(*x));

                // Do the state change very carefully. We want to trigger
                // others only when we actually need to change the state.
                if new_state != *state {
                    *state = new_state;
                }
            }
        ),
        MoveableSide::Changing { from, to } => apply_on_logic(
            logic_map, logic_q, trigger_q.iter(),
            |cond, mut state| {
                let new_state = cond.is_active(*to);

                if cond.is_active(*from) != new_state {
                    *state = TileState::Changing { to: new_state };
                }
            }
        )
    };

    if let Err(e) = res {
        error!("{}", e);
    }
}

pub fn special_tile_handler(
    mut interactions: EventReader<TileInteractionEvent>,
    mut escape_event: EventWriter<PlayerEscapedEvent>,
    mut tile_query: Query<LogicTileQuery>,
    mut move_query: Query<(MoveableQuery, Option<&PlayerTag>)>,
    mut commands: Commands,
) {
    let res: Result<(), anyhow::Error> = interactions
        .iter().try_for_each(|e| {
            let tile = tile_query.get_mut(e.tile_id)?;
            let (moveable, player_tag) = move_query.get_mut(e.moveable_id)?;

            let (mut tile, mut moveable, is_player, moveable_id, _tile_id) = (
                tile,
                moveable,
                player_tag.is_some(),
                e.moveable_id,
                e.tile_id,
            );

            match &tile.kind {
                // Button logic
                TileKind::OnceButton if is_player && !tile.is_active() =>
                    *tile.state = TileState::Ready(true),
                // Conveyor logic
                TileKind::Conveyor if tile.is_active() => {
                    moveable.slide(tile.direction(), Duration::from_millis(500));
                },
                // Frier logic
                TileKind::Frier if tile.is_active() => commands.entity(moveable_id).despawn(),
                // Spinner logic
                TileKind::Spinner if tile.is_active() => moveable
                    .rotate(tile.clock_wise(), Duration::from_millis(500)),
                // Exit logic
                TileKind::Exit if is_player && tile.is_active() => {
                    commands
                        .entity(moveable_id)
                        .remove::<MoveableBundle>()
                        .insert(PlayerWinnerTag::new());
                    escape_event.send(PlayerEscapedEvent);
                }
                _ => (),
            };

            Ok(())
        });

    if let Err(e) = res {
        error!("Error while handling tile interaction: {}", e);
    }
}

fn apply_on_logic<'a, F, I>(
    logic_map: &TileStorage,
    mut logic_q: Query<&mut TileState>,
    mut it: I,
    f: F,
) -> anyhow::Result<()>
where
    I: Iterator<Item = (&'a ActivationCondition, &'a TilePos)>,
    F: Fn(&ActivationCondition, Mut<TileState>),
{
    it.try_for_each(|(cond, pos)| {
        let state = logic_q.get_mut(
            logic_map
                .get(pos)
                .ok_or_else(|| anyhow!("Trigger at {:?} has no target", pos))?,
        )?;

        f(cond, state);

        Ok(())
    })
}