use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileEvent {
    ExitReached,
    ButtonPressed { tile_id: Entity },
}