use bevy::reflect;

#[derive(serde::Deserialize, reflect::TypeUuid)]
#[uuid = "9f19fd3b-77fa-4237-9518-95528ecb3f79"]
pub struct LevelInfo {
    level_counts: Vec<u8>,
}

impl LevelInfo {
    pub fn world_count(&self) -> u8 { self.level_counts.len() as u8 }

    pub fn level_count_in_world(&self, world: u8) -> u8 {
        self.level_counts[world as usize]
    }
}
