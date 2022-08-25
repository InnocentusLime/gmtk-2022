use bevy::prelude::*;
use super::direction::MoveDirection;
use bevy_inspector_egui::Inspectable;

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
}
