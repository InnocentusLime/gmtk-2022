use bevy::prelude::*;

pub struct TileInteractionEvent {
    pub moveable_id: Entity,
    pub tile_id: Entity,
}
