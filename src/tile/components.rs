use bevy::{prelude::*, ecs::query::WorldQuery};
use bevy_ecs_tilemap::tiles::TileFlip;
use bevy_inspector_egui::Inspectable;
use cube_rot::MoveDirection;
use serde::Deserialize;

// TODO this either needs a different named or should have the docs adjusted
/// Describes when a tile should be active, depending on what 
/// number in on player's upper side. To actually have any effect,
/// the tile needs to have [Active] attached.
#[derive(Clone, Copy, Debug, Component, Deserialize, Inspectable)]
pub enum ActivationCondition {
    /// The tile is expecting an odd number
    Odd,
    /// The tile is expecting an even number
    Even,
}

impl ActivationCondition {
    /// Tests whether the tile is active during the start of
    /// the level or not.
    pub fn active_on_start(self) -> bool {
        use crate::moveable::DecomposedRotation;

        self.is_active(DecomposedRotation::new().upper_side())
    }

    /// Tell whether the tile is active or not, assuming player
    /// has `upper_side` number on their upper side.
    pub fn is_active(self, upper_side: u8) -> bool {
        match self {
            ActivationCondition::Odd => upper_side % 2 == 1,
            ActivationCondition::Even => upper_side % 2 == 0,
        }
    }
}

/// Describes how the tile should be animated, based off its state.
#[derive(Debug, Clone, Copy, Component)]
pub enum ActivatableAnimating {
    /// The tile will switch between two different animatons, also
    /// playing a special transition animation when the state is about
    /// to switch.
    Switch {
        on_transition: usize,
        off_transition: usize,
        on_anim: usize,
        off_anim: usize,
    },
    /// The tile will simply pause its animation when it gets deactivated
    /// and will unpause it when it gets activated.
    Pause {
        anim: usize
    },
}

#[derive(Clone, Copy, Debug, Component, Inspectable, PartialEq, Eq)]
pub enum TileState {
    Ready(bool),
    Changing {
        to: bool,
    },
}

#[derive(Clone, Copy, Debug, Component, Inspectable, PartialEq, Eq, Hash, Deserialize)]
#[repr(u8)]
pub enum TileKind {
    Conveyor,
    StartTile,
    FrierTile,
    SpinningTile,
    ExitTile,
}

#[derive(WorldQuery, Debug)]
#[world_query(mutable)]
pub struct TileQuery {
    pub (super) kind: &'static TileKind,
    pub (super) state: &'static mut TileState,
    pub (super) flip: &'static TileFlip,
}

impl<'a> TileQueryItem<'a> {
    pub fn direction(&self) -> MoveDirection {
        MoveDirection::Up.apply_flipping_flags(self.flip.x, self.flip.y, self.flip.d)
    }

    pub fn clock_wise(&self) -> bool {
        !(self.flip.x ^ self.flip.y ^ self.flip.d)
    }

    pub fn is_active(&self) -> bool {
        matches!(&*self.state, TileState::Ready(true))
    }
}

// TODO start tiles shouldn't be separate type
// TODO maybe start tiles should spawn the player themselves, to break the cycle
// dependency of `player` and `tile`
/// Special floor tile, which is used as player's starting point.
#[derive(Component)]
pub struct StartTileTag;