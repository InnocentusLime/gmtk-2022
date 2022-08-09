mod components;
mod systems;

use bevy::prelude::*;
use bevy_inspector_egui::{ RegisterInspectable, InspectableRegistry };
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
        if app.world.get_resource::<InspectableRegistry>().is_some() {
            app.register_inspectable::<ActivatableTileTag>();
        }

        app
            .add_plugin(CPUTileAnimationPlugin)
            .add_stage_before(CoreStage::Update, ActiveTileSwitchStage, SystemStage::parallel())
            .add_system_set_to_stage(
                ActiveTileSwitchStage, 
                SystemSet::new()
                    .with_system(tile_switch_system)
                    .with_system(activeatable_tile_transition_system)
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(fry_logic)
                    .with_system(conveyor_logic)
                    .with_system(exit_logic)
            );
    }
}

pub fn activeatable_tile_setup(mut query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>) {
    toggle_activatable_tiles(|_| true, &mut query);
}
