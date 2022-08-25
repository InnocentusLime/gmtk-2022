use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;
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

/// Describes whether tile is active or not. This component is for tiles,
/// which are supposed to activate and deactivate during the level.
#[derive(Clone, Copy, Debug, Component, Inspectable)]
pub struct Active { 
    #[inspectable(read_only)]
    pub is_active: bool,
}

/// Conveyor tag. Conveyors push any moveable towards the direction they point at.
#[derive(Component)]
pub struct ConveyorTag;

// TODO start tiles shouldn't be separate type
// TODO maybe start tiles should spawn the player themselves, to break the cycle
// dependency of `player` and `tile`
/// Special floor tile, which is used as player's starting point.
#[derive(Component)]
pub struct StartTileTag;

/// Frier tag. Friers destroy any movable that stands on them.
#[derive(Component)]
pub struct FrierTag;

/// End tile. This tile acts as floor tile for all moveables, except the
/// player. When the player steps on this tile, the level ends.
#[derive(Component)]
pub struct EndTileTag;
