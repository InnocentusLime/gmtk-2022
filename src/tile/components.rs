use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_ecs_tilemap::tiles::TileFlip;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimation;
use bevy_inspector_egui::Inspectable;
use bevy_tiled::deserailize_from_json_str;
use cube_rot::MoveDirection;
use serde::Deserialize;

/// Describes a trigger that will activate when a button activates
#[derive(Clone, Copy, Debug, Component, Deserialize)]
#[repr(transparent)]
pub struct ButtonCondition(pub u8);

impl ButtonCondition {
    pub fn is_active(self, button_id: u8) -> bool {
        self.0 == button_id
    }
}

/// Describes a trigger that will activate based off player's side
#[derive(Clone, Copy, Debug, Component, Deserialize, Inspectable)]
pub enum SideCondition {
    /// The tile is expecting an odd number to be player's uppser side
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
#[derive(Inspectable, Debug, Default, Clone, Component, Deserialize)]
pub struct GraphicsAnimating<Anim = Handle<CPUTileAnimation>>
where
    Anim: Default,
{
    pub on_transit: Anim,
    pub off_transit: Anim,
    pub on_anim: Anim,
    pub off_anim: Anim,
}

/// Tile state. Determines what the tile would do when someone interacts with it.
#[derive(Clone, Copy, Default, Debug, Component, Inspectable, PartialEq, Eq)]
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
    Inspectable,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
)]
#[repr(u16)]
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

/// A bundle to quickly construct a logical tile.
#[derive(Clone, Default, Bundle, Deserialize)]
pub struct LogicTileBundle {
    pub ty: LogicKind,
    #[serde(skip)]
    pub state: LogicState,
}

/// A bundle to quickly construct a trigger tile.
#[derive(Clone, Bundle, Deserialize)]
pub struct TriggerTileBundle {
    pub active: SideCondition,
}

/// A bundle to quickly construct a graphics tile.
#[derive(Clone, Bundle, Deserialize)]
pub struct GraphicsTileBundle<Anim = Handle<CPUTileAnimation>>
where
    Anim: 'static + Send + Sync + Default,
{
    #[serde(
        bound = "Anim: Deserialize<'de>",
        deserialize_with = "deserailize_from_json_str"
    )]
    pub animating: GraphicsAnimating<Anim>,
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

#[derive(Component, Clone, Copy, Default)]
pub struct GraphicsTilemapTag;

#[derive(Component, Clone, Copy, Default)]
pub struct LogicTilemapTag;

#[derive(Component, Clone, Copy, Default)]
pub struct TriggerTilemapTag;
