use bevy::{prelude::*, ecs::query::WorldQuery};
use bevy_ecs_tilemap::tiles::{TileFlip, TileBundle};
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

/// Tile state. Determines what the tile would do when someone interacts with it.
#[derive(Clone, Copy, Debug, Component, Inspectable, PartialEq, Eq)]
pub enum TileState {
    /// The tile has a ready state. The bool field tells whether the tile is turned
    /// on or not.
    Ready(bool),
    /// The tile is in process of changing its state. The state will then be changed
    /// to `Ready(to)`
    Changing {
        /// The state the tile is transitioning to.
        to: bool,
    },
}

impl Default for TileState {
    fn default() -> Self { TileState::Ready(true) }
}

/// The tile kind. This data type is mapped directly to the ones you can see in the
/// level editor.
/// 
/// The tile kind dictates what graphics the game should use and how should the tile 
/// behave.
#[derive(Clone, Copy, Default, Debug, Component, Inspectable, PartialEq, Eq, Hash, Deserialize)]
#[repr(u8)]
pub enum TileKind {
    /// Conveyor tiles push any moveable into the direction they are facing towards
    /// when they are active.
    Conveyor,
    /// Start tiles are pretty much floor tiles. The only thing that makes them special
    /// is that the player spawns on them.
    Start,
    /// Frier tiles destroy any moveable that steps on them when they are active.
    Frier,
    /// Spinner tiles spin the moveable, that stepped on them, around when they are active.
    Spinner,
    /// Exit tiles announce that the level has been beated when the player-tagged moveable
    /// has stepped on them.
    Exit,
    /// Floor tiles do nothing
    #[default]
    Floor,
}

/// A bundle to quickly construct a logical tile.
#[derive(Default, Bundle)]
pub struct LogicTileBundle {
    pub kind: TileKind,
    pub state: TileState,
    #[bundle]
    pub tile_bundle: TileBundle,
}

/// A bundle to quickly construct a trigger tile.
#[derive(Bundle)]
pub struct TriggerTileBundle {
    pub condition: ActivationCondition,
    #[bundle]
    pub tile_bundle: TileBundle,
}

/// A custom query type for exposing an easier to use tile API.
#[derive(WorldQuery, Debug)]
#[world_query(mutable)]
pub struct LogicTileQuery {
    pub (super) kind: &'static TileKind,
    pub (super) state: &'static mut TileState,
    pub (super) flip: &'static TileFlip,
}

impl<'a> LogicTileQueryItem<'a> {
    /// Returns the direction towards which the tiles is facing.
    pub fn direction(&self) -> MoveDirection {
        MoveDirection::Up.apply_flipping_flags(self.flip.x, self.flip.y, self.flip.d)
    }

    /// Tells whether the tile is clock-wise or counter-clock-wise oriented.
    pub fn clock_wise(&self) -> bool {
        !(self.flip.x ^ self.flip.y ^ self.flip.d)
    }

    /// Tells whether the tile is active or not.
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

#[derive(Component)]
pub struct GraphicsTilemapTag;

#[derive(Component)]
pub struct TriggerTilemapTag;