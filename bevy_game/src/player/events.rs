use bevy::prelude::*;

use super::{ DiceRollDirection, DiceEncoding };

pub struct PlayerEscapedEvent;

pub struct PlayerMoved {
    pub cell: Entity,
    pub dice_state: DiceEncoding,
}

pub struct PlayerChangingSide {
    pub dice_state: DiceEncoding,
}
