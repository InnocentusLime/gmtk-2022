use bevy::prelude::*;

pub struct TileInteractionEvent {
    pub interactor_id: Entity,
    pub tile_id: Entity,
}
