mod components;
mod systems;
mod events;

pub use events::*;
pub use components::*;

pub use systems::*;
use bevy::prelude::*;
//use bevy_inspector_egui::{ RegisterInspectable, InspectableRegistry };

#[derive(StageLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MoveableUpdateStage;

#[derive(SystemLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum MoveableSystem {
    Tick,
    Animate,
}

#[derive(Clone, Copy, Default)]
pub struct MoveablePlugin;

impl Plugin for MoveablePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TileInteractionEvent>()
            .add_stage_after(CoreStage::Update, MoveableUpdateStage, SystemStage::parallel())
            .add_system_to_stage(MoveableUpdateStage, moveable_animation.label(MoveableSystem::Animate))
            .add_system_to_stage(MoveableUpdateStage, moveable_tick.label(MoveableSystem::Tick).before(MoveableSystem::Animate));
    }
}
