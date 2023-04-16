mod components;
mod systems;
mod events;

pub use events::*;
pub use components::*;

pub use systems::*;
use bevy::prelude::*;

#[derive(Clone, Copy, Default)]
pub struct MoveablePlugin;

impl Plugin for MoveablePlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<MoveableState>()
            .register_type::<Side>()
            .register_type::<Position>()
            .register_type::<Rotation>()
            .add_event::<TileInteractionEvent>()
            .add_system(
                moveable_tick
                    .in_base_set(CoreSet::Update)
            )
            .add_system(
                moveable_animation
                    .in_base_set(CoreSet::PostUpdate)
            );
    }
}
