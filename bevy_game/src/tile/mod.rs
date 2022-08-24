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
pub struct TileUpdateStage;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        if app.world.get_resource::<InspectableRegistry>().is_some() {
            app
                .register_inspectable::<Active>()
                .register_inspectable::<ActivationCondition>();
        }

        app
            .add_plugin(CPUTileAnimationPlugin)
            .add_stage_before(CoreStage::Update, TileUpdateStage, SystemStage::parallel())
            .add_system_set_to_stage(
                TileUpdateStage, 
                SystemSet::new()
                    .with_system(tile_state_switching)
                    .with_system(tile_animating_switch)
                    //.with_system(tile_transition_animating)
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(frier_tile_handler)
                    .with_system(conveyor_tile_handler)
                    .with_system(exit_tile_handler)
            );
    }
}
