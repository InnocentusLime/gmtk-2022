use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileEvent {
    ExitReached,
    ButtonPressed { info_entity: Entity },
}