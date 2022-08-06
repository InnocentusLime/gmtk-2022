mod components;
mod systems;

use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::{ CPUTileAnimationPlugin, CPUAnimated };
use iyes_loopless::prelude::*;

pub use components::*;

use systems::*;

use crate::states::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StageLabel)]
pub struct ActiveTileSwitchStage;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(CPUTileAnimationPlugin)
            .add_stage_before(CoreStage::Update, ActiveTileSwitchStage, SystemStage::parallel())
            .add_system_to_stage(ActiveTileSwitchStage, tile_switch_system.run_in_state(GameState::InGame))
            .add_system_to_stage(ActiveTileSwitchStage, activeatable_tile_transition_system.run_in_state(GameState::InGame))
            .add_system(fry_logic)
            .add_system(conveyor_logic)
            .add_system(exit_logic);
    }
}

pub fn activeatable_tile_setup(mut query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>) {
    toggle_activatable_tiles(|_| true, &mut query);
}
