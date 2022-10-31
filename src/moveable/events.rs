use bevy::prelude::*;

/// This event gets created by the moveable update system, when the a moveable
/// finishes its animation.
pub struct TileInteractionEvent {
    pub moveable_id: Entity,
    pub tile_id: Entity,
}
