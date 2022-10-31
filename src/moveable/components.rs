use bevy::{prelude::*, ecs::query::WorldQuery};
use bevy_ecs_tilemap::prelude::*;
use std::time::Duration;

pub use cube_rot::*;

/// Tracks moveable's rotation. This component has not public
/// API and is used by the systems internally.
#[derive(Debug, Clone, Copy, Default, Component)]
#[repr(transparent)]
pub struct Rotation(pub (super) DecomposedRotation);

/// Tracks moveable's position. This component has not public
/// API and is used by the systems internally.
#[derive(Debug, Clone, Copy, Default, Component)]
#[repr(transparent)]
pub struct Position(pub (super) TilePos);

/// The type of the move. 
#[derive(Debug, Clone, Copy)]
pub enum MoveTy {
    /// The object will move into some direction, potentially also
    /// changing its side.
    Slide {
        dir: MoveDirection,
        next_pos: TilePos,
    },
    /// The object is rotating around its orthogonal axis.
    Rotate {
        clock_wise: bool,
    }
}

/// Moveables current state. While the fields of that component are
/// exposed, it is not recommened to change the data of this component
/// directly and instead use the API provided by [MoveableQuery].
#[derive(Debug, Clone, Default, Component)]
pub enum MoveableState {
    #[default]
    Idle,
    Moving {
        timer: Timer,
        ty: MoveTy,
    },
}

/// Public-API component for tracking the upper side of a moveable.
/// This component triggers the `Changed<..>` filter **only** when the
/// side of the moveable actually changes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Component)]
pub enum Side {
    /// The moveable isn't changing its side. The number is encoding
    /// the ID of that number.
    Ready(u8),
    /// The moveable is changing its side `from` something `to` something.
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

/// A bundle to conveniently add all all the components, that usually belong
/// to a moveable. 
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

/// A wrapper-query type, which exposes an easy-to-use API for
/// moveables.
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct MoveableQuery {
    pub(super) position: &'static mut Position,
    pub(super) rotation: &'static mut Rotation,
    pub(super) side: &'static mut Side,
    pub(super) state: &'static mut MoveableState,
}

impl<'a> MoveableQueryItem<'a> {
    /// Forefully resets moveable's state to `Idle`. 
    /// 
    /// **WARNING:** this will announce to other systems, that
    /// the side of the moveable has been set to `Ready(..)`. This
    /// is to ensure, that the moveable stays in a correct state.
    pub(super) fn force_idle(&mut self) {
        *self.side = Side::Ready(self.rotation.0.upper_side());
        *self.state = MoveableState::Idle;
    }

    /// Asks the game to slide the moveable in some direction (See [MoveableState::Moving]).
    /// 
    /// Note that this won't announce to other systems that the moveable has
    /// started changing its side.
    /// 
    /// Note that all this method does is **asking** the game to do that. The
    /// game might actually deny the request.
    pub fn slide(&mut self, dir: MoveDirection, time: Duration) {
        self.try_slide(dir, time);
    }

    /// Asks the game to flip the moveable in some direction (See [MoveableState::Moving]).
    /// 
    /// Note that this won't announce to other systems that the moveable has
    /// started changing its side.
    /// 
    /// Note that all this method does is **asking** the game to do that. The
    /// game might actually deny the request.
    pub fn flip(&mut self, dir: MoveDirection, time: Duration) {
        if self.try_slide(dir, time) {
            *self.side = Side::Changing { 
                from: self.rotation.0.upper_side(), 
                to: self.rotation.0.rotate_in_dir(dir).upper_side(), 
            };
        }
    }

    /// Asks the game rotate the moveable clockwise or couterclockwise
    /// 
    /// Note that all this method does is **asking** the game to do that. The
    /// game might actually deny the request.
    pub fn rotate(&mut self, clock_wise: bool, time: Duration) {
        self.try_set_state(MoveableState::Moving { 
            timer: Timer::new(time, false), 
            ty: MoveTy::Rotate { clock_wise }, 
        });
    }

    fn try_slide(&mut self, dir: MoveDirection, time: Duration) -> bool {
        let next_pos = match dir.apply_on_pos(self.position.0) {
            Some(x) => x,
            None => return false,
        };

        self.try_set_state(MoveableState::Moving { 
            timer: Timer::new(time, false), 
            ty: MoveTy::Slide { dir, next_pos }, 
        })
    }

    /// Sets the state to `new_state` as long as `self.state` is `Idle`.
    fn try_set_state(&mut self, state: MoveableState) -> bool {
        if !matches!(&*self.state, MoveableState::Idle) { return false; }
        *self.state = state;

        true
    }
}