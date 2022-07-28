mod components;
mod systems;

use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::{ CPUTileAnimationPlugin, CPUAnimated };
use iyes_loopless::prelude::*;

pub use components::*;

use systems::*;

use crate::states::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StageLabel)]
pub struct ActiveTileUpdateStage;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(CPUTileAnimationPlugin)
            .add_stage_before(CoreStage::Update, ActiveTileUpdateStage, SystemStage::parallel())
            .add_system_to_stage(ActiveTileUpdateStage, tile_switch_system.run_in_state(GameState::InGame))
            .add_system_to_stage(ActiveTileUpdateStage, activeatable_tile_transition_system.run_in_state(GameState::InGame));
        add_tile::<ConveyorTag>(app);
        add_tile::<FrierTag>(app);
        add_tile::<StartTileTag>(app);
        add_tile::<EndTileTag>(app);
        add_tile::<SolidTileTag>(app);
    }
}
    
fn add_tile<T: TileState>(app: &mut App) {
    app.add_system(tile_reaction_system::<T>.run_in_state(GameState::InGame));
}

pub fn activeatable_tile_setup(query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>) {
    toggle_activatable_tiles(|_| true, query);
}
