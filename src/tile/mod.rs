mod components;
mod systems;
mod events;

use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUTileAnimationPlugin;

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
        app
            .add_plugin(CPUTileAnimationPlugin)
            .register_type::<ButtonCondition>()
            .register_type::<GraphicsAnimating>()
            .register_type::<LogicState>()
            .register_type::<LogicKind>()
            .register_type::<GraphicsAnimating>()
            .register_type::<SideCondition>()
            .add_event::<TileEvent>()
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
