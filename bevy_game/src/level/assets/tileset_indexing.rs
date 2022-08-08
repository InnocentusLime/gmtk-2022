use std::collections::HashMap;

pub enum TilesetIndexing {
    Continious,
    Special(HashMap<u32, u32>),
}

impl TilesetIndexing {
    pub fn dispatch(&self, x: u32) -> u32 {
        match self {
            TilesetIndexing::Continious => x,
            TilesetIndexing::Special(map) => map[&x],
        }
    }
}
