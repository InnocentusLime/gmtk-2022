use bevy::prelude::Resource;
use std::collections::HashSet;

use crate::level_info::LevelInfo;

#[derive(Resource, Clone, Debug, serde::Serialize, serde::Deserialize)]
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

    /*
    pub fn give_achievement(&mut self, name: &str) { self.achievements.insert(name.to_owned()); }

    pub fn has_achievement(&self, name: &str) -> bool { self.achievements.contains(name) }
    */

    pub fn register_level_complete(&mut self, info: &LevelInfo) {
        if let Some((world, level)) = self.next_level(info) {
            self.world = world;
            self.level = level;
        }
    }

    pub fn next_level(&self, info: &LevelInfo) -> Option<(u8, u8)> {
        let (mut world, mut level) = (self.world, self.level);

        if world >= info.world_count() { return None; }

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
}
