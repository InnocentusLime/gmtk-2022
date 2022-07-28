mod activatable_tile_data;
mod tile_state;

use bevy::prelude::*;
use crate::player::{ PlayerModification, DiceEncoding, DiceRollDirection };

pub use activatable_tile_data::*;
pub use tile_state::*;

#[derive(Component)]
pub struct ActivatableTileTag {
    state: bool,
    pub condition: ActivationCondition,
    pub anim_info: ActivatableAnimating,
}

impl ActivatableTileTag {
    pub fn new(
        condition: ActivationCondition,
        anim_info: ActivatableAnimating,
    ) -> Self {
        ActivatableTileTag {
            state: condition.active_on_start(),
            condition, anim_info,
        }
    }

    pub fn is_active(&self) -> bool { self.state }

    pub fn will_be_active(&self, player_state: &DiceEncoding) -> bool {
        self.condition.is_active(player_state)
    }

    /// Updated the state of the tile, returning `true` if the internal
    /// logic needs to be updated.
    pub(super) fn update(&mut self, player_state: &DiceEncoding) -> bool {
        let new_state = self.will_be_active(player_state);
        let result = self.state != new_state;
        self.state = new_state;
        result
    }
}

#[derive(Component)]
pub struct ConveyorTag;

impl TileState for ConveyorTag {
    type UpdateData = &'static ActivatableTileTag;

    fn react<'w, 's>(&mut self, extra: &ActivatableTileTag) -> PlayerModification {
        if extra.is_active() {
            PlayerModification::Slide(DiceRollDirection::Up)
        } else {
            PlayerModification::AcknowledgeMove
        }
    }
}

#[derive(Component)]
pub struct SolidTileTag;

impl TileState for SolidTileTag {
    type UpdateData = ();

    fn react<'w, 's>(&mut self, _extra: ()) -> PlayerModification {
        PlayerModification::AcknowledgeMove
    }
}

#[derive(Component)]
pub struct StartTileTag;

impl TileState for StartTileTag {
    type UpdateData = ();

    fn react<'w, 's>(&mut self, _extra: ()) -> PlayerModification {
        PlayerModification::AcknowledgeMove
    }
}

#[derive(Component)]
pub struct FrierTag;

impl TileState for FrierTag {
    type UpdateData = &'static ActivatableTileTag;

    fn react<'w, 's>(&mut self, extra: &ActivatableTileTag) -> PlayerModification {
        if extra.is_active() {
            PlayerModification::Kill
        } else {
            PlayerModification::AcknowledgeMove
        }
    }
}

#[derive(Component)]
pub struct EndTileTag;

impl TileState for EndTileTag {
    type UpdateData = ();

    fn react<'w, 's>(&mut self, _extra: ()) -> PlayerModification {
        PlayerModification::Escape
    }
}
