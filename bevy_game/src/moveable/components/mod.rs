mod direction;
mod moveable_state;
mod decomposed_rotation;

use bevy::prelude::*;
use std::time::Duration;
pub(super) use moveable_state::*;
pub(super) use decomposed_rotation::*;

pub use direction::MoveDirection;
pub use decomposed_rotation::DecomposedRotation;

#[derive(Debug, Clone, Component)]
pub struct Moveable {
    pub(super) pos: (u32, u32),
    pub(super) rot: DecomposedRotation,
    pub(super) state: MoveableState,
}

impl Moveable {
    /// Create a moveable placed on `pos`.
    pub fn new(pos: (u32, u32)) -> Self {
        Self {
            pos,
            state: MoveableState::Idle,
            rot: DecomposedRotation::new(),
        }
    }

    /// Gets the current rotation of a moveable.
    #[inline]
    pub fn rotation(&self) -> DecomposedRotation { self.rot }

    /// Gets the current position of a moveable.
    #[inline]
    pub fn pos(&self) -> (u32, u32) { self.pos }

    /// Starts the rolling animation. The animation will play exactly
    /// `time` long.
    /// The call will have no effect if there's an animation in progress.
    /// Returns `true` if the command was successive, returns `false` if the
    /// entity is already doing an animation.
    #[inline]
    pub fn flip(&mut self, dir: MoveDirection, time: Duration) -> bool {
        match &self.state {
            MoveableState::Idle => { self.state = MoveableState::Moving {
                timer: Timer::new(time, false),
                dir,
                ty: MoveTy::Flip,
            }; true },
            _ => false,
        }
    }

    /// Starts the sliding animation. The animation will play exactly
    /// `time` long.
    /// The call will have no effect if there's an animation in progress.
    /// Returns `true` if the command was successive, returns `false` if the
    /// entity is already doing an animation.
    #[inline]
    pub fn slide(&mut self, dir: MoveDirection, time: Duration) -> bool {
        match &self.state {
            MoveableState::Idle => { self.state = MoveableState::Moving {
                timer: Timer::new(time, false),
                dir,
                ty: MoveTy::Slide,
            }; true },
            _ => false,
        }
    }

    /// Force-sets moveable's state to `Idle`
    pub fn force_idle(&mut self) {
        self.state = MoveableState::Idle;
    }

    /// Returns the cell the moveable is going to occupy at the end of
    /// the animation. Returns `None` if no animation is playing.
    #[inline]
    pub fn going_to_occupy(&self) -> Option<(u32, u32)> {
        match &self.state {
            MoveableState::Idle => None,
            MoveableState::Moving { dir, .. } => dir.apply_on_pos(self.pos),
        }
    }
}
