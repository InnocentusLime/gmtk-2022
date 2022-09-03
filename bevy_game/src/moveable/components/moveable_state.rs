use bevy::prelude::*;
use super::direction::MoveDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoveTy {
    #[default]
    Slide,
    Flip,
}

#[derive(Debug, Clone, Default)]
pub enum MoveableState {
    #[default]
    Idle,
    Moving {
        timer: Timer,
        dir: MoveDirection,
        ty: MoveTy,
        just_started: bool,
    },
    Rotating {
        timer: Timer,
        clock_wise: bool,
    },
}
