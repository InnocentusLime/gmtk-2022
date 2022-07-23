use bevy::log::warn;

use std::collections::HashSet;

use crate::level_info::LevelInfo;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Save {
    achievements: HashSet<String>,
    world: u8, // The world player has yet to beat
    level: u8, // The level the player has yet to beat
}

impl Save {
    pub fn new() -> Self {
        Save {
            achievements: HashSet::new(),
            world: 0,
            level: 0,
        }
    }

    pub fn world_level(&self) -> (u8, u8) { (self.world, self.level) }

    pub fn give_achievement(&mut self, name: &str) { self.achievements.insert(name.to_owned()); }

    pub fn has_achievement(&self, name: &str) -> bool { self.achievements.contains(name) }

    pub fn register_level_complete(&mut self, info: &LevelInfo) {
        if let Some((world, level)) = self.next_level(info) {
            self.world = world;
            self.level = level; 
        }
    }

    pub fn next_level(&self, info: &LevelInfo) -> Option<(u8, u8)> {
        let (mut world, mut level) = (self.world, self.level);

        if world as usize >= info.world_count() { return None; }

        // Check if we have beaten the last level of the world
        if info.level_count_in_world(world) - 1 <= level {
            level = 0;
            world += 1;
        } else {
            level += 1;
        }

        if world == info.world_count() {
            None
        } else {
            Some((world, level))
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn load() -> Result<Self, anyhow::Error> {
        use std::fs::File;
        Ok(serde_json::from_reader(File::open("save.json")?)?)
    }
    
    #[cfg(target_arch = "wasm")]
    pub fn load() -> Result<Self, anyhow::Error> {
        compile_error!("Loading saves hasn't been implemented for wasm yet");
        todo!() 
    }

    #[cfg(target_arch = "x86_64")]
    pub fn save(&self) -> Result<(), anyhow::Error> {
        use std::fs::File;
        Ok(serde_json::to_writer(File::create("save.json")?, &self)?)
    }
    
    #[cfg(target_arch = "wasm")]
    pub fn save(&self) -> Result<(), anyhow::Error> {
        compile_error!("Saving hasn't been implemented for wasm yet");
        todo!() 
    }
}
