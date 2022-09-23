mod components;
mod systems;

use bevy::prelude::*;
use bevy_inspector_egui::{ RegisterInspectable, InspectableRegistry };
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimationPlugin;

pub use components::*;

use systems::*;

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
                //.register_inspectable::<Active>()
                .register_inspectable::<ActivationCondition>();
        }

        app
            .add_plugin(CPUTileAnimationPlugin)
            .add_stage_before(CoreStage::Update, TileUpdateStage, SystemStage::parallel())
            .add_system_set_to_stage(
                TileUpdateStage, 
                SystemSet::new()
                    .with_system(tile_state_switching.label(TileSystem::StateSwitch))
                    .with_system(tile_animation_switch.label(TileSystem::AnimationSwitch).after(TileSystem::TransitionAnimation))
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(special_tile_handler)
            );
    }
}
