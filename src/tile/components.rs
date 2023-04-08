use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_ecs_tilemap::tiles::TileFlip;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimation;
use cube_rot::MoveDirection;

/// Describes a trigger that will activate when a button activates
#[derive(Clone, Copy, Debug, Component, Default, Reflect)]
#[repr(transparent)]
#[reflect(Component)]
pub struct ButtonCondition(pub u8);

impl ButtonCondition {
    pub fn is_active(self, button_id: u8) -> bool {
        self.0 == button_id
    }
}

/// Describes a trigger that will activate based off player's side
#[derive(Clone, Copy, Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub enum SideCondition {
    /// The tile is expecting an odd number to be player's uppser side
    #[default]
    OnOddSide,
    /// The tile is expecting an even number to be player's upper side
    OnEvenSide,
}

impl SideCondition {
    pub fn is_active(self, upper_side: u8) -> bool {
        match self {
            SideCondition::OnOddSide => upper_side % 2 == 1,
            SideCondition::OnEvenSide => upper_side % 2 == 0,
        }
    }
}

/// Describes how the tile should be animated, based off its state.
#[derive(Reflect, Debug, Default, Clone, Component)]
#[reflect(Component)]
pub struct GraphicsAnimating {
    pub on_transit: Handle<CPUTileAnimation>,
    pub off_transit: Handle<CPUTileAnimation>,
    pub on_anim: Handle<CPUTileAnimation>,
    pub off_anim: Handle<CPUTileAnimation>,
}

/// Tile state. Determines what the tile would do when someone interacts with it.
#[derive(Clone, Copy, Default, Debug, Component, Reflect, PartialEq, Eq)]
#[reflect(Component)]
pub struct LogicState(pub bool);

/// The tile kind. This data type is mapped directly to the ones you can see in the
/// level editor.
///
/// The tile kind dictates what graphics the game should use and how should the tile
/// behave.
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    Component,
    Reflect,
    PartialEq,
    Eq,
    Hash,
)]
#[repr(u16)]
#[reflect(Component)]
pub enum LogicKind {
    /// Once button tiles are activated when interacted with by a player and stay activated
    /// for the rest of the level.
    OnceButton(u8),
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

/// A custom query type for exposing an easier to use tile API.
#[derive(WorldQuery, Debug)]
#[world_query(mutable)]
pub struct LogicTileQuery {
    pub(super) kind: &'static LogicKind,
    pub(super) state: &'static mut LogicState,
    pub(super) flip: &'static TileFlip,
}

impl<'a> LogicTileQueryItem<'a> {
    /// Returns the direction towards which the tiles is facing.
    pub fn direction(&self) -> MoveDirection {
        MoveDirection::Up.apply_flipping_flags(
            self.flip.x,
            self.flip.y,
            self.flip.d,
        )
    }

    /// Tells whether the tile is clock-wise or counter-clock-wise oriented.
    pub fn is_clock_wise(&self) -> bool {
        !(self.flip.x ^ self.flip.y ^ self.flip.d)
    }

    /// Tells whether the tile is active or not.
    pub fn is_active(&self) -> bool {
        self.state.0
    }
}