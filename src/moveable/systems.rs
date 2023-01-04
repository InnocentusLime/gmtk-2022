use std::time::Duration;
use super::events::*;
use super::components::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub const MOVEABLE_Z_POS: f32 = 101f32;

/// Updates the internals of a moveable. Returns an iteraction event if one should
/// be posted.
fn update_moveable(
    moveable_id: Entity,
    item: &mut MoveableQueryItem,
    tiles: &TileStorage,
    dt: Duration,
) -> Option<TileInteractionEvent> {
    match &mut *item.state {
        // No interactions when idle
        MoveableState::Idle => None,
        // Moving objects might attempt interacting
        MoveableState::Moving { timer, ty } => match &ty {
            // If we are rotating -- we don't interact with the tile
            MoveTy::Rotate { clock_wise } => {
                // Update the rotation and become idle once the timer ticks down
                if timer.tick(dt).just_finished() {
                    item.rotation.0 = item.rotation.0.rotate_ortho(*clock_wise);
                    *item.state = MoveableState::Idle;
                }

                None
            },
            // If we are actually changing our current tile, we will interact
            MoveTy::Slide { dir, next_pos } => {
                // Verify the presence of the tile and the validity of the ID
                let tile_id = match tiles.get(next_pos) {
                    Some(x) => x,
                    None => {
                        item.force_idle();
                        return None;
                    },
                };

                // When we have finished moving, update our pos and rotation (if needed)
                if timer.tick(dt).just_finished() {
                    // If we were changing our side, finish it
                    if let Side::Changing { to, .. } = *item.side {
                        item.rotation.0 = item.rotation.0.rotate_in_dir(*dir);
                        *item.side = Side::Ready(to);
                    }

                    item.position.0 = *next_pos;
                    *item.state = MoveableState::Idle;

                    Some(TileInteractionEvent { moveable_id, tile_id })
                } else {
                    None
                }
            },
        }
    }
}

// TODO implement collisions to later implement boxes?
/// Updates the state data of all moveables. This system does the following:
///
/// * Cancels all attempts to move (force-transitioning all moveables into `Idle` style)
/// if they are trying to move into a tile that doesn't exist.
/// * Tick all timers on moveables.
/// * Issue `TileInteractionEvent` when a moveable is done moving onto a tile.
/// * Performs state transitions on the moveables when they are done moving.
pub fn moveable_tick(
    mut interaction_events: EventWriter<TileInteractionEvent>,
    mut moveable_q: Query<(Entity, MoveableQuery)>,
    map_q: Query<&TileStorage, With<MoveableTilemapTag>>,
    time: Res<Time>,
) {
    let dt = time.delta();
    let tiles = match map_q.get_single() {
        Ok(x) => x,
        Err(_) => return,
    };

    moveable_q.for_each_mut(|(id, mut moveable)| {
        if let Some(e) = update_moveable(id, &mut moveable, tiles, dt) {
            interaction_events.send(e);
        }
    });
}

// TODO Maybe doing animations this way is bad
/// Animates all moveables. This state simply alters moveable's transform to
/// create the animations.
pub fn moveable_animation(
    map_q: Query<(&Transform, &TilemapGridSize), With<MoveableTilemapTag>>,
    mut moveable_q: Query<(&mut Transform, MoveableQuery), Without<MoveableTilemapTag>>,
) {
    use crate::level::tile_pos_to_world_pos;

    if map_q.get_single().is_err() { return; }
    let (map_tf, map_grid) = map_q.single();

    moveable_q.for_each_mut(|(mut tf, moveable)| {
        let current_pos = tile_pos_to_world_pos(moveable.position.0, map_tf, map_grid);

        match &*moveable.state {
            MoveableState::Moving { timer, ty } => {
                let t = timer.percent();

                match &ty {
                    MoveTy::Slide { dir, next_pos } => {
                        let start_pos = current_pos;
                        let end_pos = tile_pos_to_world_pos(*next_pos, map_tf, map_grid);

                        // Animate the sliding
                        tf.translation = (start_pos + (end_pos - start_pos) * t).extend(MOVEABLE_Z_POS);

                        // Add some rotation if we are changing the side
                        if matches!(&*moveable.side, Side::Changing { .. }) {
                            tf.rotation = dir.to_quat(t) * moveable.rotation.0.rot_quat();
                        }
                    },
                    MoveTy::Rotate { clock_wise } => {
                        tf.rotation = if *clock_wise {
                            Quat::from_rotation_z(-t * std::f32::consts::FRAC_PI_2)
                        } else {
                            Quat::from_rotation_z(t * std::f32::consts::FRAC_PI_2)
                        } * moveable.rotation.0.rot_quat();
                    }
                }
            },
            MoveableState::Idle => {
                tf.translation = current_pos.extend(MOVEABLE_Z_POS);
                tf.rotation = moveable.rotation.0.rot_quat();
            },
        }
    });
}
