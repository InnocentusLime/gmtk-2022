use crate::app::GameplayCamera;
use super::dice;
use super::{ Player, PlayerState, PlayerModification, MoveInfo, Direction, PlayerMoved, PlayerChangingSide };

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::level::tile_pos_to_world_pos;
use bevy::input::keyboard::KeyboardInput;

pub fn player_update(
    time: Res<Time>,
    mut events: EventReader<PlayerModification>,
    mut player_q: Query<(&Transform, &mut Player, &mut PlayerState)>,
    mut map_q: MapQuery,
    map_entity: Query<&Transform, With<Map>>,
    mut move_event: EventWriter<PlayerMoved>,
    mut changing_side: EventWriter<PlayerChangingSide>, 
) {
    let map_tf = map_entity.single();
    let (tf, mut pl, mut st) = player_q.single_mut();

    // Event based state transitions
    for e in events.iter() {
        match (e, &mut *st) {
            (PlayerModification::AcknowledgeMove, PlayerState::AwaitingAcknowledge { .. }) => *st = PlayerState::AwaitingInput,
            (PlayerModification::Roll(dir), PlayerState::AwaitingAcknowledge { .. } | PlayerState::AwaitingInput) => {
                let (nx, ny) = pl.next_cell(dir.to_offset());
                changing_side.send(PlayerChangingSide {
                    dice_state: pl.apply_rotation(*dir),
                });
                match map_q.get_tile_entity(TilePos(nx, ny), pl.map_id, pl.layer_id) {
                    Ok(to_entity) => *st = PlayerState::Moving {
                        to_entity,
                        to: (nx, ny),
                        start_pos: tf.translation,
                        end_pos: tile_pos_to_world_pos((nx, ny), map_tf, &mut map_q, pl.map_id, pl.layer_id).extend(1.0f32),
                        timer: Timer::from_seconds(0.52, false),
                        info: MoveInfo::Rotate {
                            direction: *dir,
                            start_rot: tf.rotation,
                        },
                    },
                    Err(e) => warn!("Can't roll player in dir: {:?}\nReason:{}", dir, e),
                }
            },
            (PlayerModification::Slide(dir), PlayerState::AwaitingAcknowledge { .. } | PlayerState::AwaitingInput) => {
                let (nx, ny) = pl.next_cell(dir.to_offset());
                match map_q.get_tile_entity(TilePos(nx, ny), pl.map_id, pl.layer_id) {
                    Ok(to_entity) => *st = PlayerState::Moving {
                        to_entity,
                        to: (nx, ny),
                        start_pos: tf.translation,
                        end_pos: tile_pos_to_world_pos((nx, ny), map_tf, &mut map_q, pl.map_id, pl.layer_id).extend(1.0f32),
                        timer: Timer::from_seconds(0.5, false),
                        info: MoveInfo::Slide,
                    },
                    Err(e) => warn!("Can't slide player in dir: {:?}\nReason:{}", dir, e),
                }
            },
            (PlayerModification::Kill, PlayerState::AwaitingAcknowledge { .. }) => (),
            (PlayerModification::Escape, PlayerState::AwaitingAcknowledge { .. }) => *st = PlayerState::Escaping {
                timer: Timer::from_seconds(0.31, false),
            },
            _ => error!("Modfication:\n{:?}\ndoesn't fit player's state\n{:?}", e, st),
        }

        return;
    }

    // Other state transitions
    match &mut *st {
        PlayerState::Escaping { timer } => {
            timer.tick(time.delta());
        },
        PlayerState::Moving { end_pos, to, timer, info, to_entity, .. } => {
            timer.tick(time.delta());
            if timer.finished() {
                match info {
                    MoveInfo::Slide => (),
                    MoveInfo::Rotate { direction, .. } => {
                        pl.state = pl.apply_rotation(*direction);
                        trace!("New side: {}", pl.upper_side());
                    },
                }
                pl.current_cell = *to;
                move_event.send(PlayerMoved {
                    cell: *to_entity,
                    dice_state: pl.state,
                });
                *st = PlayerState::AwaitingAcknowledge {
                    new_rot: pl.state.rot_quat(),
                    new_pos: *end_pos,
                };
            }
        },
        _ => (),
    }
}

pub fn player_animation(
    //mut player_q: Query<(&mut Transform, &PlayerState)>,
    mut col_mats: ResMut<Assets<ColorMaterial>>,
    mut player_q: Query<(&mut Transform, &PlayerState, &Player, &mut Handle<ColorMaterial>)>,
) {
    let (mut player_tf, st, pl, mat_handle) = player_q.single_mut();
    match st {
        PlayerState::Moving { start_pos, end_pos, timer, info, .. } => {
            let t = 1.0f32 - timer.percent_left();
            match info {
                MoveInfo::Slide => {
                    player_tf.translation = *start_pos + (*end_pos - *start_pos) * t;
                },
                MoveInfo::Rotate { direction, start_rot } => {
                    let mut t = t;
                    match pl.picked_anim {
                        0 => (),
                        1 => t = ((2.0f32 * t - 1.0f32).powi(3) + 1.0f32) / 2.0f32,
                        2 => t = 2.0 * t.powi(3) - t,
                        _ => error!("Weird anim ID"),
                    }
                    player_tf.translation = *start_pos + (*end_pos - *start_pos) * t;
                    player_tf.rotation = direction.to_quat(t) * *start_rot
                },
            }
        },
        PlayerState::AwaitingAcknowledge { new_pos, new_rot } => {
            player_tf.rotation = *new_rot;
            player_tf.translation = *new_pos;
        },
        PlayerState::Escaping { timer } => {
            let t = timer.percent_left();
            // TODO hardcoded player size
            player_tf.scale = Vec3::new(25.0f32, 25.0f32, 25.0f32) * t;
            col_mats.get_mut(&*mat_handle).unwrap().color = Color::Rgba {
                red: t,
                green: t,
                blue: t,
                alpha: 1.0,
            };
        },
        PlayerState::AwaitingInput => (),
    }
}

pub fn player_camera(
    mut player_q: Query<&mut Transform, (With<Player>, Without<GameplayCamera>)>,
    mut gameplay_camera: Query<&mut Transform, With<GameplayCamera>>,
) {
    for player_tf in player_q.iter_mut() {
        gameplay_camera.single_mut().translation = player_tf.translation.truncate().extend(50.0f32);
    }
}

pub fn player_controls(
    mut events: EventReader<KeyboardInput>,
    mut output: EventWriter<PlayerModification>,
//    query: Query<&PlayerState>,
    mut query: Query<(&PlayerState, &mut Player)>,
) {
    use bevy::input::ElementState;
    use Direction::*;
   
    // TODO pretify?
    let mut movement = None;
    for ev in events.iter() {
        if movement.is_some() { continue; }
        if ev.state != ElementState::Pressed { continue; }

        for (st, mut pl) in query.iter_mut() {
            if !matches!(*st, PlayerState::AwaitingInput) { continue; }
            match ev.key_code { 
                Some(KeyCode::Space) => {
                    pl.picked_anim = (pl.picked_anim + 1) % 3;
                    info!("Anim mode: {}", pl.picked_anim);
                },
                Some(KeyCode::W) => movement = Some(Up),
                Some(KeyCode::A) => movement = Some(Left),
                Some(KeyCode::S) => movement = Some(Down),
                Some(KeyCode::D) => movement = Some(Right),
                _ => (),
            }
        }
    }

    if let Some(movement) = movement {
        output.send(PlayerModification::Roll(movement));
    }
}
