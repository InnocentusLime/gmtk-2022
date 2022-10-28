mod components;
mod systems;

use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimationPlugin;
use bevy_inspector_egui::{InspectableRegistry, RegisterInspectable};

pub use components::*;

use systems::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StageLabel)]
pub struct TileUpdateStage;

pub struct TilePlugin;

#[derive(Clone, Copy, SystemLabel)]
enum TileSystem {
    TileUpdate,
    TileSetup,
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
                //.register_inspectable::<ActivatableAnimating>()
                .register_inspectable::<ActivationCondition>();
        }

        app.add_plugin(CPUTileAnimationPlugin)
            .add_stage_before(
                CoreStage::Update,
                TileUpdateStage,
                SystemStage::parallel(),
            )
            .add_system_set_to_stage(
                TileUpdateStage,
                SystemSet::new()
                    .with_system(
                        tile_animation_setup.label(TileSystem::TileSetup),
                    )
                    .with_system(
                        tile_state_switching
                            .label(TileSystem::StateSwitch)
                            .after(TileSystem::TileSetup),
                    )
                    .with_system(
                        tile_animation_switch
                            .label(TileSystem::AnimationSwitch)
                            .after(TileSystem::StateSwitch),
                    )
                    .with_system(
                        special_tile_handler
                            .label(TileSystem::TileSetup)
                    ),
            );
    }
}
