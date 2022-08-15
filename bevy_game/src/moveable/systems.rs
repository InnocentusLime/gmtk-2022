use std::time::Duration;
use super::events::*;
use super::components::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::level::LevelTag;

/// Updated the internals of this component. Returns `true` if the moveable
/// ended up changing its tile.
fn update_moveable(moveable: &mut Moveable, dt: Duration) -> bool {
    match &mut moveable.state {
        MoveableState::Idle => false,
        MoveableState::Moving { timer, dir, ty, .. } if timer.finished() => {
            match dir.apply_on_pos(moveable.pos) {
                Some(pos) => moveable.pos = pos,
                None => error!("Failed to change moveble position"),
            }
                
            if *ty == MoveTy::Flip {
                moveable.rot = moveable.rot.rotate_in_dir(*dir);
            }
                
            moveable.state = MoveableState::Idle;

            true
        },
        MoveableState::Moving { timer, .. } => { timer.tick(dt); false },
    }
}

/// Updates the state data of all moveables.
pub fn moveable_tick(
    mut interaction_events: EventWriter<TileInteractionEvent>,
    mut moveable_q: Query<(Entity, &mut Moveable)>,
    map_q: Query<&TileStorage, With<LevelTag>>,
    time: Res<Time>,
) {
    let dt = time.delta();

    moveable_q.for_each_mut(|(id, mut moveable)| {
        let tiles = match map_q.get_single() {
            Ok(x) => x,
            Err(_) => return,
        };

        if let Some(pos) = moveable.going_to_occupy() {
            if tiles.get(&pos).is_none() {
                moveable.force_idle();
            }
        }

        if update_moveable(&mut *moveable, dt) {
            match tiles.get(&moveable.pos()) {
                Some(tile_id) => interaction_events.send(TileInteractionEvent {
                    tile_id,
                    interactor_id: id,
                }),
                None => error!("Entity {id:?} has ended up on an illegal tiles pos ({:?}).", moveable.pos()),
            }
        } 
    });
}

/// Animates all moveables.
pub fn moveable_animation(
    map_q: Query<(&Transform, &TilemapGridSize), With<LevelTag>>,
    mut moveable_q: Query<(&mut Transform, &Moveable), Without<LevelTag>>,
) {
    use crate::level::tile_pos_to_world_pos;

    if map_q.get_single().is_err() { return; }
    let (map_tf, map_grid) = map_q.single();

    moveable_q.for_each_mut(|(mut tf, moveable)| {
        let current_pos = tile_pos_to_world_pos(moveable.pos(), map_tf, map_grid).extend(1.0f32);

        // Other animation curves for moving
        // t = ((2.0f32 * t - 1.0f32).powi(3) + 1.0f32) / 2.0f32
        // t = 2.0 * t.powi(3) - t
        match (&moveable.state, moveable.going_to_occupy()) {
            (MoveableState::Moving { timer, ty, dir, .. }, Some(n_pos)) => {
                let t = 1.0f32 - timer.percent_left();
                let start_pos = current_pos;
                let end_pos = tile_pos_to_world_pos(n_pos, map_tf, map_grid).extend(1.0f32);
                
                tf.translation = start_pos + (end_pos - start_pos) * t;
            
                if *ty == MoveTy::Flip {
                    tf.rotation = dir.to_quat(t) * moveable.rotation().rot_quat();
                }
            },
            (MoveableState::Idle, _) => {
                tf.translation = current_pos;
                tf.rotation = moveable.rotation().rot_quat();
            },
            _ => (),
        }
    });
}
