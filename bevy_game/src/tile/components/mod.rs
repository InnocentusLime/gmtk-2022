mod activatable_tile_data;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::player::{ DiceEncoding };

pub use activatable_tile_data::*;

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

    pub fn will_be_active(&self, side: u8) -> bool {
        self.condition.is_active(side)
    }

    /// Updated the state of the tile, returning `true` if the internal
    /// logic needs to be updated.
    pub(super) fn update(&mut self, side: u8) -> bool {
        let new_state = self.will_be_active(side);
        let result = self.state != new_state;
        self.state = new_state;
        result
    }
}

#[derive(Component)]
pub struct ConveyorTag;

#[derive(Component)]
pub struct StartTileTag;

#[derive(Component)]
pub struct FrierTag;

#[derive(Component)]
pub struct EndTileTag;
