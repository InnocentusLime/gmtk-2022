use bevy::{prelude::*, ecs::query::WorldQuery};
use bevy_ecs_tilemap::prelude::*;
use std::time::Duration;

pub use cube_rot::*;

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Rotation(pub (super) DecomposedRotation);

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Position(pub (super) TilePos);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoveTy {
    #[default]
    Slide,
    Flip,
}

#[derive(Debug, Clone, Default, Component)]
pub enum MoveableState {
    #[default]
    Idle,
    Moving {
        timer: Timer,
        dir: MoveDirection,
        ty: MoveTy,
    },
    Rotating {
        timer: Timer,
        clock_wise: bool,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Component)]
pub enum Side {
    Ready(u8),
    Changing {
        from: u8,
        to: u8,
    },
}

impl Default for Side {
    fn default() -> Self {
        Side::Ready(Rotation::default().0.upper_side())
    }
}

#[derive(Clone, Debug, Default, Bundle)]
pub struct MoveableBundle {
    pub rotation: Rotation,
    pub position: Position,
    pub side: Side,
    pub state: MoveableState,
}

impl MoveableBundle {
    pub fn new(position: TilePos) -> Self {
        MoveableBundle { 
            position: Position(position),
            ..default()
        }
    }
}

/// Tag for tilemap, which moveables are intended to traverse.
#[derive(Default, Clone, Copy, Debug, Component)]
pub struct MoveableTilemapTag;

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct MoveableQuery {
    pub(super) position: &'static mut Position,
    pub(super) rotation: &'static mut Rotation,
    pub(super) side: &'static mut Side,
    pub(super) state: &'static mut MoveableState,
}

impl<'a> MoveableQueryItem<'a> {
    pub(super) fn force_idle(&mut self) {
        *self.side = Side::Ready(self.rotation.0.upper_side());
        *self.state = MoveableState::Idle;
    }

    pub fn slide(&mut self, dir: MoveDirection, time: Duration) {
        *self.state = MoveableState::Moving { 
            dir,
            timer: Timer::new(time, false), 
            ty: MoveTy::Slide, 
        };
    }

    pub fn flip(&mut self, dir: MoveDirection, time: Duration) {
        *self.side = Side::Changing { 
            from: self.rotation.0.upper_side(), 
            to: self.rotation.0.rotate_in_dir(dir).upper_side(), 
        };
        *self.state = MoveableState::Moving { 
            dir,
            timer: Timer::new(time, false), 
            ty: MoveTy::Flip, 
        };
    }

    pub fn rotate(&mut self, clock_wise: bool, time: Duration) {
        *self.state = MoveableState::Rotating { 
            clock_wise, 
            timer: Timer::new(time, false), 
        };
    }
}