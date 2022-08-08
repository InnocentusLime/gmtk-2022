use bevy::prelude::*;
use super::direction::MoveDirection;
use bevy_inspector_egui::Inspectable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoveTy {
    #[default]
    Slide,
    Flip,
}

#[derive(Debug, Clone, Default, Inspectable)]
pub enum MoveableState {
    #[default]
    Idle,
    Moving {
        #[inspectable(ignore)]
        timer: Timer,
        #[inspectable(ignore)]
        dir: MoveDirection,
        #[inspectable(ignore)]
        ty: MoveTy,
    },
}
