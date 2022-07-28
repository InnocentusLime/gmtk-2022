use bevy::prelude::*;

#[derive(Component)]
pub struct LevelInfo {
    pub(super) map: u16,
    pub(super) geometry_layer: u16,
}

impl LevelInfo {
    pub fn map(&self) -> u16 { self.map }

    pub fn geometry_layer(&self) -> u16 { self.geometry_layer }
}
