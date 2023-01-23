mod components;
mod systems;
mod events;

use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimationPlugin;
use bevy_inspector_egui::{InspectableRegistry, RegisterInspectable};

pub use components::*;

use systems::*;

use crate::moveable::MoveableUpdateStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StageLabel)]
pub struct TileUpdateStage;

pub struct TilePlugin;

#[derive(Clone, Copy, SystemLabel)]
enum TileSystem {
    TileUpdate,
    StateSwitch,
    AnimationSwitch,
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        if app.world.get_resource::<InspectableRegistry>().is_some() {
            app
                //.register_inspectable::<Active>()
                .register_inspectable::<TileState>()
                .register_inspectable::<TileKind>()
                .register_inspectable::<ActivatableAnimating>()
                .register_inspectable::<ActivationCondition>();
        }

        app.add_plugin(CPUTileAnimationPlugin)
            .add_stage_after(
                MoveableUpdateStage,
                TileUpdateStage,
                SystemStage::parallel(),
            )
            .add_system_set_to_stage(
                TileUpdateStage,
                SystemSet::new()
                    .with_system(
                        tile_state_switching
                            .label(TileSystem::StateSwitch),
                    )
                    .with_system(
                        tile_animation_switch
                            .label(TileSystem::AnimationSwitch)
                            .after(TileSystem::StateSwitch),
                    )
                    .with_system(
                        special_tile_handler
                            .label(TileSystem::TileUpdate)
                            .after(TileSystem::StateSwitch)
                    ),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                tile_animation_setup
            );
    }
}
