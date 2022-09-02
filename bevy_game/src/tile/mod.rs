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

#[derive(Clone, Copy, SystemLabel)]
enum TileSystem {
    StateSwitch,
    AnimationSwitch,
    TransitionAnimation,
}

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
                    .with_system(tile_state_switching.label(TileSystem::StateSwitch))
                    .with_system(tile_transition_animating.label(TileSystem::TransitionAnimation).after(TileSystem::StateSwitch))
                    .with_system(tile_animating_switch.label(TileSystem::AnimationSwitch).after(TileSystem::TransitionAnimation))
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(frier_tile_handler)
                    .with_system(conveyor_tile_handler)
                    .with_system(exit_tile_handler)
                    .with_system(spinning_tile_handler)
            );
    }
}
