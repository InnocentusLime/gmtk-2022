use std::collections::HashMap;

/// A type, which encodes mapping from `Tiled` tile IDs to
/// engine's IDs in the tile atlas.
#[derive(Debug)]
pub enum TilesetIndexing {
    Continious,
    Special(HashMap<u32, u32>),
}

impl TilesetIndexing {
    /// Maps the tile ID from `Tiled` to the engine's tile ID.
    pub fn dispatch(&self, x: u32) -> u32 {
        match self {
            TilesetIndexing::Continious => x,
            TilesetIndexing::Special(map) => map[&x],
        }
    }
}
