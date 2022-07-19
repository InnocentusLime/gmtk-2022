use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::player::PlayerModification;
use super::tile_def::*;

pub struct SpecialTilePlugin;

impl Plugin for SpecialTilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_stage_before(CoreStage::Update, ActiveTileUpdateStage, SystemStage::parallel())
            .add_activatable_tile::<ConveyorTag>()
            .add_activatable_tile::<FrierTag>()
            .add_tile::<StartTileTag>()
            .add_tile::<EndTileTag>()
            .add_tile::<SolidTileTag>();
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

impl ActivatableTileState for ConveyorTag {
    type SwitchData = (Entity, &'static mut Tile);
    
    fn enable<'w, 's>(&mut self, commands: &mut Commands, (me, _): (Entity, Mut<Tile>)) {
        commands.entity(me).insert(GPUAnimated::new(0, 4, 8.0f32));
    }
    
    fn disable<'w, 's>(&mut self, commands: &mut Commands, (me, mut tile_dat): (Entity, Mut<Tile>)) {
        commands.entity(me).remove::<GPUAnimated>();
        // FIXME EWWW HARDCODED TILES
        tile_dat.texture_index = 0;
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

// FIXME EWWW HARDCODED TILES
impl ActivatableTileState for FrierTag {
    type SwitchData = &'static mut Tile;
    
    fn enable<'w, 's>(&mut self, _commands: &mut Commands, mut tile_dat: Mut<Tile>) {
        tile_dat.texture_index = 8;
    }
    
    fn disable<'w, 's>(&mut self, _commands: &mut Commands, mut tile_dat: Mut<Tile>) {
        tile_dat.texture_index = 4;
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
