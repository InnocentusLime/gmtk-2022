use std::time::Duration;
use super::events::*;
use super::components::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

/// Updates the internals of a moveable. Returns `true` if the moveable
/// reached the destination (if it was in process of moving from one position to another).
fn update_moveable(
    item: &mut MoveableQueryItem,
    dt: Duration,
) -> bool {
    match &mut *item.state {
        MoveableState::Idle => false,
        MoveableState::Moving { timer, dir, ty, .. } if timer.finished() => {
            match dir.apply_on_pos(item.position.0) {
                Some(new_pos) => item.position.0 = new_pos,
                None => error!("Failed to change moveble position"),
            }
                
            if *ty == MoveTy::Flip {
                item.rotation.0 = item.rotation.0.rotate_in_dir(*dir);
            }
                
            *item.state = MoveableState::Idle;
            *item.side = Side::Ready(item.rotation.0.upper_side());

            true
        },
        MoveableState::Moving { timer, .. } => { 
            timer.tick(dt); 

            false 
        },
        MoveableState::Rotating { timer, clock_wise } if timer.finished() => {
            item.rotation.0 = item.rotation.0.rotate_ortho(*clock_wise);
            *item.state = MoveableState::Idle;

            false
        },
        MoveableState::Rotating { timer, .. } => {
            timer.tick(dt);

            false
        },
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

    moveable_q.for_each_mut(|(id, mut moveable)| {
        let tiles = match map_q.get_single() {
            Ok(x) => x,
            Err(_) => return,
        };

        if let MoveableState::Moving { dir, .. } = &*moveable.state {
            if dir.apply_on_pos(moveable.position.0).map(|x| tiles.get(&x)).is_none() {
                moveable.force_idle();
            }
        }

        if update_moveable(&mut moveable, dt) {
            match tiles.get(&moveable.position.0) {
                Some(tile_id) => interaction_events.send(TileInteractionEvent {
                    tile_id,
                    moveable_id: id,
                }),
                None => error!("Entity {id:?} has ended up on an illegal tiles pos ({:?}).", moveable.position.0),
            }
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
        let current_pos = tile_pos_to_world_pos(moveable.position.0, map_tf, map_grid).extend(1.0f32);

        match &*moveable.state {
            MoveableState::Moving { timer, dir, ty } => {
                let t = 1.0f32 - timer.percent_left();
                if let Some(n_pos) = dir.apply_on_pos(moveable.position.0) {
                    let start_pos = current_pos;
                    let end_pos = tile_pos_to_world_pos(n_pos, map_tf, map_grid).extend(1.0f32);
                    
                    tf.translation = start_pos + (end_pos - start_pos) * t;
                    if *ty == MoveTy::Flip {
                        tf.rotation = dir.to_quat(t) * moveable.rotation.0.rot_quat();
                    }
                }
            },
            MoveableState::Idle => {
                tf.translation = current_pos;
                tf.rotation = moveable.rotation.0.rot_quat();
            },
            MoveableState::Rotating { timer, clock_wise } => {
                let t = 1.0f32 - timer.percent_left();

                tf.rotation = if *clock_wise {
                        Quat::from_rotation_z(-t * std::f32::consts::FRAC_PI_2)
                    } else {
                        Quat::from_rotation_z(t * std::f32::consts::FRAC_PI_2)
                    } * moveable.rotation.0.rot_quat();
            },
        }
    });
}
