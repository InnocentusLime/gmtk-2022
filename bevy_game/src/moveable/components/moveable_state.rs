use bevy::prelude::*;
use super::direction::MoveDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTy {
    Slide,
    Flip,
}

#[derive(Debug, Clone)]
pub enum MoveableState {
    Idle,
    Moving {
        timer: Timer,
        dir: MoveDirection,
        ty: MoveTy,
    },
}
