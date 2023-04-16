mod components;
mod systems;
mod events;

use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::{CPUTileAnimationPlugin, update_animation_frames};

pub use components::*;
pub use events::*;
pub use systems::*;

use crate::moveable::moveable_tick;

#[derive(Clone, Copy, Debug, SystemSet, PartialEq, Eq, Hash)]
pub enum TileSystems {
    Triggers,
    InteractionHandler,
}

pub struct TilePlugin;

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
            .add_systems(
                (
                    handle_button_triggers,
                    handle_player_side_triggers,
                )
                .after(moveable_tick)
                .in_set(TileSystems::Triggers)
            )
            .add_system(
                special_tile_handler
                .in_set(TileSystems::InteractionHandler)
                .after(TileSystems::Triggers)
            )
            .add_systems(
                (
                    tile_state_animation_switch,
                    tile_transition_anim_switch,
                )
                .in_base_set(CoreSet::PostUpdate)
                .before(update_animation_frames)
            );
    }
}
