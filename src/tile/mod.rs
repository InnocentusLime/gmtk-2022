mod components;
mod systems;
mod events;

use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimationPlugin;
use bevy_inspector_egui::{InspectableRegistry, RegisterInspectable};

pub use components::*;
pub use events::*;
pub use systems::*;

use crate::moveable::MoveableUpdateStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StageLabel)]
pub struct TileUpdateStage;

pub struct TilePlugin;

#[derive(Clone, Copy, SystemLabel)]
enum TileSystem {
    TileUpdate,
    SideTrigger,
    ButtonTrigger,
    AnimationSwitch,
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        if app.world.get_resource::<InspectableRegistry>().is_some() {
            app
                .register_inspectable::<LogicState>()
                .register_inspectable::<LogicKind>()
                .register_inspectable::<GraphicsAnimating>()
                .register_inspectable::<SideCondition>()
                .add_event::<TileEvent>();
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
                        handle_player_side_triggers
                            .label(TileSystem::SideTrigger),
                    )
                    .with_system(
                        handle_button_triggers
                            .label(TileSystem::ButtonTrigger),
                    )
                    .with_system(
                        special_tile_handler
                            .label(TileSystem::TileUpdate)
                            .after(TileSystem::SideTrigger)
                            .after(TileSystem::ButtonTrigger),
                    )
                    .with_system(
                        tile_state_animation_switch
                            .label(TileSystem::AnimationSwitch)
                            .after(TileSystem::SideTrigger)
                            .after(TileSystem::ButtonTrigger),
                    )
                    .with_system(
                        tile_transition_anim_switch
                            .after(TileSystem::AnimationSwitch),
                    ),
            );
    }
}
