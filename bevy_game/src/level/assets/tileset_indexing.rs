use std::collections::HashMap;

pub enum TilesetIndexing {
    Continious,
    Special(HashMap<u32, u16>),
}

impl TilesetIndexing {
    pub fn dispatch(&self, x: u32) -> u16 {
        match self {
            TilesetIndexing::Continious => x as u16,
            TilesetIndexing::Special(map) => map[&x],
        }
    }
}
