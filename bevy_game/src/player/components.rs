use bevy::prelude::*;

use super::{ DiceRollDirection, DiceEncoding };

#[derive(Debug, Clone)]
pub enum MoveInfo {
    Slide,
    Rotate {
        start_rot: Quat,
        direction: DiceRollDirection,
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
    pub picked_anim: u8,
    map_id: u16, // The map ID the player traverses
    layer_id: u16, // The layer ID 
    // TODO state -> dice_state
    pub state: DiceEncoding,
    pub current_cell: (u32, u32),
}

impl Player {
    pub fn new(start: (u32, u32), map_id: u16, layer_id: u16) -> Self {
        Player {
            picked_anim: 0,
            map_id,
            layer_id,
            current_cell: start,
            state: DiceEncoding::new(),
        }
    }

    pub fn apply_rotation(&mut self, dir: DiceRollDirection) -> DiceEncoding { self.state.apply_rotation(dir) }

    pub fn upper_side(&self) -> u8 { self.state.upper_side() }

    pub fn next_cell(&self, off: (i32, i32)) -> (u32, u32) {
        // FIXME MIGHT CRASH
        (
            (self.current_cell.0 as i32 + off.0) as u32,
            (self.current_cell.1 as i32 + off.1) as u32
        )
    }

    pub fn map_id(&self) -> u16 { self.map_id }

    pub fn layer_id(&self) -> u16 { self.layer_id }
}
