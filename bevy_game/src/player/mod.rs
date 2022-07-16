mod dice;

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::level::tile_pos_to_world_pos;

use crate::level::Level;
use crate::app::GameplayCamera;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(player_controls)
            .add_system(player_camera)
            /*.register_type::<PlayerState>()*/;
    }
}

#[derive(Clone, Component)]
pub struct PlayerState {
    state: dice::DiceEncoding,
    current_cell: (u32, u32),
}

impl PlayerState {
    pub fn new(start: (u32, u32)) -> Self {
        PlayerState {
            current_cell: start,
            state: dice::DiceEncoding::new(),
        }
    }

    pub fn apply_rotation(&mut self, dir: dice::Direction) { self.state.apply_rotation(dir) }

    pub fn upper_side(&self) -> u8 { self.state.upper_side() }
}

pub fn player_camera(
    mut player_q: Query<(&mut Transform, &PlayerState), (Without<Map>, Without<GameplayCamera>)>,
    mut gameplay_camera: Query<&mut Transform, (Without<Map>, With<GameplayCamera>)>,
    mut map: MapQuery,
    map_entity: Query<&Transform, With<Map>>,
) {
    for (mut player_tf, player) in player_q.iter_mut() {
        for map_tf in map_entity.iter() {
            // TODO retrieve geometry layer id
            let pos = tile_pos_to_world_pos(player.current_cell, map_tf, &mut map, 0, 0);
            player_tf.translation = pos.extend(1.0f32);
            gameplay_camera.single_mut().translation = pos.extend(1.0f32);
        }
    }
}

pub fn player_controls(
    mut events: EventReader<KeyboardInput>,
    mut query: Query<&mut PlayerState>,
    mut map: MapQuery,
) {
    use bevy::input::ElementState;
    use dice::Direction::*;
    
    let mut movement = None;
    for ev in events.iter() {
        if movement.is_some() { continue; }
        if ev.state != ElementState::Pressed { continue; }

        for mut state in query.iter_mut() {
            match ev.key_code { 
                Some(KeyCode::W) => movement = Some(((0, 1), Up)),
                Some(KeyCode::A) => movement = Some(((-1, 0), Left)),
                Some(KeyCode::S) => movement = Some(((0, -1), Down)),
                Some(KeyCode::D) => movement = Some(((1, 0), Right)),
                _ => (),
            }
        }

    }

    // TODO do not process here
    for mut state in query.iter_mut() {
        if let Some(((dx, dy), dir)) = movement {
            let (nx, ny) = (
                    (state.current_cell.0 as i32 + dx) as u32,
                    (state.current_cell.1 as i32 + dy) as u32,
                );
            // TODO player should know the layer
            let has_tile = map.get_tile_entity(
                TilePos(nx, ny),
                0,
                0,
            ).is_ok();
                
            if has_tile {
                info!("New pos! ({}, {})", nx, ny);
                state.apply_rotation(dir);
                state.current_cell = (nx, ny);
                info!("Your side is {}", state.upper_side());
            }
        }    
    }
}
