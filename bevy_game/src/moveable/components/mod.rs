mod direction;
mod moveable_state;
mod decomposed_rotation;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::time::Duration;
use bevy_inspector_egui::Inspectable;
pub(super) use moveable_state::*;
pub(super) use decomposed_rotation::*;

pub use direction::MoveDirection;
pub use decomposed_rotation::DecomposedRotation;

/// Contains all information about a moveable:
/// 
/// * Rotation
/// * Position
/// * State
///
/// `Transform` **does not** encode the actual position of a moveable.
/// It's actual position is whatever returned by `pos()` method.
#[derive(Debug, Clone, Component, Default)]
pub struct Moveable {
    pub(super) pos: TilePos,
    pub(super) rot: DecomposedRotation,
    pub(super) state: MoveableState,
}

impl Moveable {
    /// Create a moveable placed on `pos`.
    pub fn new(pos: TilePos) -> Self {
        Self {
            pos,
            state: MoveableState::Idle,
            rot: DecomposedRotation::new(),
        }
    }

    /// Asks if the moveable has started moving this
    /// very frame.
    #[inline]
    pub fn just_started_moving(&self) -> bool {
        match &self.state {
            MoveableState::Moving { just_started, .. } => *just_started,
            _ => false,
        }
    }

    /// Gets the current upper side of a moveable
    pub fn upper_side(&self) -> u8 { self.rot.upper_side() }

    /// Gets the current rotation of a moveable.
    #[inline]
    pub fn rotation(&self) -> DecomposedRotation { self.rot }

    /// Gets the current position of a moveable.
    #[inline]
    pub fn pos(&self) -> TilePos { self.pos }

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
                just_started: true,
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
                just_started: true,
            }; true },
            _ => false,
        }
    }

    /// Force-sets moveable's state to `Idle`
    pub fn force_idle(&mut self) {
        self.state = MoveableState::Idle;
    }

    /// Returns the cell the moveable is going to occupy at the end of
    /// the animation. Returns `None` if no position changing animation is playing.
    #[inline]
    pub fn going_to_occupy(&self) -> Option<TilePos> {
        match &self.state {
            MoveableState::Idle => None,
            MoveableState::Moving { dir, .. } => dir.apply_on_pos(self.pos),
        }
    }

    /// Returns the side that the moveable will have up once its animation
    /// finished. Returns `None` if no side changing animation is playing.
    #[inline]
    pub fn next_side(&self) -> Option<u8> {
        match &self.state {
            MoveableState::Moving { dir, ty, .. } if *ty == MoveTy::Flip => {
                Some(self.rot.rotate_in_dir(*dir).upper_side())
            }, 
            _ => None,
        }
    }
}

/// Tag for tilemap, which moveables are intended to traverse.
#[derive(Default, Clone, Copy, Debug, Component)]
pub struct MoveableTilemapTag;
