pub mod dice;

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::level::tile_pos_to_world_pos;
use iyes_loopless::prelude::*;

use crate::states::GameState;
use crate::app::GameplayCamera;

pub use dice::Direction;

#[derive(StageLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct PlayerUpdateStage;

#[derive(SystemLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum PlayerSystems {
    Control,
    Update,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayerMoved>()
            .add_event::<PlayerChangingSide>()
            .add_event::<PlayerModification>()
            .add_stage_after(
                CoreStage::PreUpdate,
                PlayerUpdateStage,
                SystemStage::parallel()
            )
            .add_system_to_stage(
                PlayerUpdateStage,
                player_controls.run_in_state(GameState::InGame).label(PlayerSystems::Control)
            )
            .add_system_to_stage(
                PlayerUpdateStage,
                player_update.run_in_state(GameState::InGame).label(PlayerSystems::Update)
                    .after(PlayerSystems::Control)
            )
            .add_system_set_to_stage(
                PlayerUpdateStage,
                ConditionSet::new()
                    .after(PlayerSystems::Update).run_in_state(GameState::InGame)
                    .with_system(player_animation).with_system(player_camera).into()
            )
            /*.register_type::<PlayerState>()*/;
    }
}

#[derive(Debug)]
pub enum PlayerModification {
    AcknowledgeMove,
    Roll(dice::Direction),
    Slide(dice::Direction),
    Kill,
    Escape,
}

pub struct PlayerMoved {
    pub cell: Entity,
    pub dice_state: dice::DiceEncoding,
}

pub struct PlayerChangingSide {
    pub dice_state: dice::DiceEncoding,
}

#[derive(Debug, Clone)]
pub enum MoveInfo {
    Slide,
    Rotate {
        start_rot: Quat,
        direction: dice::Direction,
    },
}

#[derive(Debug, Clone, Component)]
pub enum PlayerState {
    AwaitingInput,
    Moving {
        to: (u32, u32),
        to_entity: Entity,
        start_pos: Vec3,
        end_pos: Vec3,
        timer: Timer,
        info: MoveInfo,
    },
    AwaitingAcknowledge {
        new_pos: Vec3,
        new_rot: Quat,
    },
    Escaping {
        timer: Timer,
    },
}

#[derive(Clone, Component)]
pub struct Player {
    picked_anim: u8,
    map_id: u16, // The map ID the player traverses
    layer_id: u16, // The layer ID 
    // TODO state -> dice_state
    state: dice::DiceEncoding,
    current_cell: (u32, u32),
}

impl Player {
    pub fn new(start: (u32, u32), map_id: u16, layer_id: u16) -> Self {
        Player {
            picked_anim: 0,
            map_id,
            layer_id,
            current_cell: start,
            state: dice::DiceEncoding::new(),
        }
    }

    pub fn apply_rotation(&mut self, dir: dice::Direction) -> dice::DiceEncoding { self.state.apply_rotation(dir) }

    pub fn upper_side(&self) -> u8 { self.state.upper_side() }

    pub fn next_cell(&self, off: (i32, i32)) -> (u32, u32) {
        // FIXME MIGHT CRASH
        (
            (self.current_cell.0 as i32 + off.0) as u32,
            (self.current_cell.1 as i32 + off.1) as u32
        )
    }
}

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
