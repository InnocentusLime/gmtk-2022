pub mod dice;

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::level::tile_pos_to_world_pos;
use iyes_loopless::prelude::*;

use crate::states::GameState;
use crate::level::Level;
use crate::app::GameplayCamera;

#[derive(SystemLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct PlayerControlsLabel;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_to_stage(
                CoreStage::PreUpdate, 
                player_controls.run_in_state(GameState::InGame)
                .label(PlayerControlsLabel)
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(player_animation.run_in_state(GameState::InGame))
                    .with_system(player_camera.run_in_state(GameState::InGame)) 
                    .with_system(player_states.run_in_state(GameState::InGame))
                    .with_system(player_movement.run_in_state(GameState::InGame))
                    .after(PlayerControlsLabel)
            )
            .add_event::<PlayerMoved>()
            .add_event::<MovePlayer>()
            .add_event::<PlayerMoveAcknowledged>()
            /*.register_type::<PlayerState>()*/;
    }
}

pub struct MovePlayer {
    pub spin: bool,
    pub dir: dice::Direction,
}

pub struct PlayerMoved {
    pub cell: Entity,
    pub dice_state: dice::DiceEncoding,
}

pub struct PlayerMoveAcknowledged;

// TODO implement animations procedurally
#[derive(Clone, Component)]
pub enum PlayerState {
    AwaitingInput,
    Sliding {
        to: (u32, u32),
        start_pos: Vec3,
        timer: Timer,
    },
    Rolling {
        to: (u32, u32),
        direction: dice::Direction,
        start_rot: Quat,
        start_pos: Vec3,
        timer: Timer,
    },
    AwaitingAcknowledge,
}

#[derive(Clone, Component)]
pub struct Player {
    // TODO state -> dice_state
    state: dice::DiceEncoding,
    current_cell: (u32, u32),
}

impl Player {
    pub fn new(start: (u32, u32)) -> Self {
        Player {
            current_cell: start,
            state: dice::DiceEncoding::new(),
        }
    }

    pub fn apply_rotation(&mut self, dir: dice::Direction) { self.state.apply_rotation(dir) }

    pub fn upper_side(&self) -> u8 { self.state.upper_side() }
}

pub fn player_states(
    mut ack_events: EventReader<PlayerMoveAcknowledged>,
    mut player_q: Query<&mut PlayerState>,
) {
    for e in ack_events.iter() {
        for mut st in player_q.iter_mut() {
            match &mut *st {
                PlayerState::AwaitingAcknowledge => *st = PlayerState::AwaitingInput,
                _ => continue,
            }
        }
    }
}

pub fn player_animation(
    time: Res<Time>,
    mut move_event: EventWriter<PlayerMoved>,
    mut map: MapQuery,
    map_entity: Query<&Transform, With<Map>>,
    mut player_q: Query<(&mut Transform, &mut Player, &mut PlayerState), Without<Map>>,
) {
    for (mut player_tf, mut player, mut st) in player_q.iter_mut() {
        match &mut *st {
            PlayerState::Rolling { to, direction, start_rot, start_pos, timer } => {
                timer.tick(time.delta());
                for map_tf in map_entity.iter() {
                    let world_pos = tile_pos_to_world_pos(*to, map_tf, &mut map, 0, 0).extend(1.0f32);
                    // TODO retrieve geometry layer id
                    let t = 1.0f32 - timer.percent_left();
                    player_tf.translation = *start_pos + (world_pos - *start_pos) * t;
                    // TODO this stacks rounding errors
                    match direction {
                        dice::Direction::Left => player_tf.rotation = Quat::from_rotation_y(-t * std::f32::consts::FRAC_PI_2) * *start_rot,
                        dice::Direction::Right => player_tf.rotation = Quat::from_rotation_y(t * std::f32::consts::FRAC_PI_2) * *start_rot,
                        dice::Direction::Up => player_tf.rotation = Quat::from_rotation_x(-t * std::f32::consts::FRAC_PI_2) * *start_rot,
                        dice::Direction::Down => player_tf.rotation = Quat::from_rotation_x(t * std::f32::consts::FRAC_PI_2) * *start_rot,
                    }
                }

                if timer.finished() {
                    // TODO state transitions should be handled by someone else
                    player.current_cell = *to;
                    player.state.apply_rotation(*direction);
                    info!("New side: {}", player.state.upper_side());
                    move_event.send(PlayerMoved {
                        // TODO crash = bad
                        cell: map.get_tile_entity(TilePos(to.0, to.1), 0, 0).unwrap(),
                        dice_state: player.state,
                    });
                    *st = PlayerState::AwaitingAcknowledge;
                }
            },
            PlayerState::Sliding { to, start_pos, timer } => {
                timer.tick(time.delta());
                for map_tf in map_entity.iter() {
                    let world_pos = tile_pos_to_world_pos(*to, map_tf, &mut map, 0, 0).extend(1.0f32);
                    let t = 1.0f32 - timer.percent_left();
                    player_tf.translation = *start_pos + (world_pos - *start_pos) * t;
                }

                if timer.finished() {
                    // TODO state transitions should be handled by someone else
                    player.current_cell = *to;
                    move_event.send(PlayerMoved {
                        // TODO crash = bad
                        cell: map.get_tile_entity(TilePos(to.0, to.1), 0, 0).unwrap(),
                        dice_state: player.state,
                    });
                    *st = PlayerState::AwaitingAcknowledge;
                }
            },
            PlayerState::AwaitingInput => {
                for map_tf in map_entity.iter() {
                    let world_pos = tile_pos_to_world_pos(player.current_cell, map_tf, &mut map, 0, 0).extend(1.0f32);
                    // TODO retrieve geometry layer id
                    player_tf.translation = world_pos;
                }
            },
            _ => continue,
        };
    }
}

pub fn player_camera(
    mut player_q: Query<&mut Transform, (With<Player>, Without<GameplayCamera>)>,
    mut gameplay_camera: Query<&mut Transform, With<GameplayCamera>>,
) {
    for mut player_tf in player_q.iter_mut() {
        gameplay_camera.single_mut().translation = player_tf.translation.truncate().extend(50.0f32);
    }
}

// TODO merge with "player_states"
pub fn player_movement(
    mut events: EventReader<MovePlayer>,
    mut query: Query<(&Transform, &mut Player, &mut PlayerState)>,
    mut map: MapQuery,
) {
    use bevy::input::ElementState;
    use dice::Direction::*;
    
    let mut movement = None;
    for ev in events.iter() {
        for (_, _, st) in query.iter_mut() {
            match ev.dir { 
                Up => movement = Some(((0, 1), Up, ev.spin)),
                Left => movement = Some(((-1, 0), Left, ev.spin)),
                Down => movement = Some(((0, -1), Down, ev.spin)),
                Right => movement = Some(((1, 0), Right, ev.spin)),
                _ => (),
            }
        }
    }

    for (tf, mut pl, mut state) in query.iter_mut() {
        if let Some(((dx, dy), dir, spin)) = movement {
           let (nx, ny) = (
                    (pl.current_cell.0 as i32 + dx) as u32,
                    (pl.current_cell.1 as i32 + dy) as u32,
                );
            
            // TODO player should know the layer
            let has_tile = map.get_tile_entity(
                TilePos(nx, ny),
                0,
                0,
            ).is_ok();
                
            if has_tile {
                if spin {
                    *state = PlayerState::Rolling {
                        to: (nx, ny),
                        direction: dir,
                        start_rot: tf.rotation,
                        start_pos: tf.translation,
                        timer: Timer::from_seconds(0.8, false),
                    };
                } else {
                    *state = PlayerState::Sliding {
                        to: (nx, ny),
                        start_pos: tf.translation,
                        timer: Timer::from_seconds(0.5, false),
                    };
                }
            }
        }    
    }
}

pub fn player_controls(
    mut events: EventReader<KeyboardInput>,
    mut output: EventWriter<MovePlayer>,
    mut query: Query<&PlayerState>,
) {
    use bevy::input::ElementState;
    use dice::Direction::*;
    
    let mut movement = None;
    for ev in events.iter() {
        if movement.is_some() { continue; }
        if ev.state != ElementState::Pressed { continue; }

        for st in query.iter() {
            if !matches!(*st, PlayerState::AwaitingInput) { continue; }
            match ev.key_code { 
                Some(KeyCode::W) => movement = Some(Up),
                Some(KeyCode::A) => movement = Some(Left),
                Some(KeyCode::S) => movement = Some(Down),
                Some(KeyCode::D) => movement = Some(Right),
                _ => (),
            }
        }
    }

    if let Some(movement) = movement {
        output.send(MovePlayer {
            spin: true,
            dir: movement,
        });
    }
}
