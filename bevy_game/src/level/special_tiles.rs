use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::states::GameState;
use crate::player::PlayerModification;
use super::tile_def::*;

pub struct SpecialTilePlugin;

impl Plugin for SpecialTilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_stage_before(CoreStage::Update, ActiveTileUpdateStage, SystemStage::parallel())
            .add_tile::<ConveyorTag>()
            .add_tile::<FrierTag>()
            .add_tile::<StartTileTag>()
            .add_tile::<EndTileTag>()
            .add_tile::<SolidTileTag>()
            .add_system_to_stage(ActiveTileUpdateStage, tile_switch_system.run_in_state(GameState::InGame))
            .add_system(
                activeatable_tile_setup_system.run_in_state(GameState::Spawning)
                .run_on_event::<MapReady>()
            );
    }
}

#[derive(Component)]
pub struct ConveyorTag;

impl TileState for ConveyorTag {
    type UpdateData = &'static ActivatableTileTag;

    fn react<'w, 's>(&mut self, extra: &ActivatableTileTag) -> PlayerModification {
        // TODO player should re-import Direction
        use crate::player::dice::Direction;

        if extra.is_active() {
            PlayerModification::Slide(Direction::Up)
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
