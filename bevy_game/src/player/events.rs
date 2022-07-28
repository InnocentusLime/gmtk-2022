use bevy::prelude::*;

use super::{ DiceRollDirection, DiceEncoding };

#[derive(Debug)]
pub enum PlayerModification {
    AcknowledgeMove,
    Roll(DiceRollDirection),
    Slide(DiceRollDirection),
    Kill,
    Escape,
}

pub struct PlayerMoved {
    pub cell: Entity,
    pub dice_state: DiceEncoding,
}

pub struct PlayerChangingSide {
    pub dice_state: DiceEncoding,
}
