use anyhow::{ Context, anyhow, bail };
use bevy_asset_loader::asset_collection::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap_cpu_anim::*;
use bevy_tiled::*;
use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use std::collections::HashMap;
use thiserror::Error;
use serde::Deserialize;
use bevy_tiled::{ TiledMap, TilesetIndexing };

use crate::tile::{ ActivationCondition, ActivatableAnimating };

#[derive(AssetCollection)]
pub struct BaseLevelAssets {
    #[asset(key = "map")]
    pub map: Handle<TiledMap>,
}

#[derive(AssetCollection)]
pub struct LevelTilesetImages {
    #[asset(key = "tileset_images", collection(typed))]
    pub images: Vec<Handle<TextureAtlas>>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum TileAnimationType {
    OnOffAnimation,
    OnAnimation,
    OffAnimation,
    OnTransitionAnimation,
    OffTransitionAnimation,
}

#[derive(Deserialize)]
struct LevelTileAnimation {
    anim_ty: TileAnimationType,
    target: LevelTileType,
}

#[derive(Deserialize)]
struct ActivatorTile {
    active: ActivationCondition,
}

#[derive(Deserialize)]
enum GeometryTile {
    LevelTile(LevelTile),
    Frame,
    LevelTileAnimation(LevelTileAnimation),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[repr(u8)]
pub enum LevelTileType {
    Conveyor,
    Fry,
    PlayerStart,
    Floor,
}

impl LevelTileType {
    pub fn insert_into(&self, cmds: &mut EntityCommands) {
        use crate::tile::{ FrierTag, ConveyorTag, StartTileTag };
                            
        match self {
            LevelTileType::Fry => { cmds.insert(FrierTag); },
            LevelTileType::Conveyor => { cmds.insert(ConveyorTag); },
            LevelTileType::PlayerStart => { cmds.insert(StartTileTag); },
            _ => (),
        }
    }
}

#[derive(Deserialize)]
struct LevelTile {
    ty: LevelTileType,
}

static GEOMETRY_LAYER_ID: &'static str = "geometry";
static ACTIVATOR_LAYER_ID: &'static str = "activators";

// TODO the level manages graphics, which doesn't
// seem to be a good idea in the end.
pub struct Level {
    width: u32,
    height: u32,
    pub(super) geometry_atlas: Handle<Image>,
    pub(super) geometry_graphics: HashMap<(u32, u32), u32>,
    pub(super) activators: HashMap<(u32, u32), ActivationCondition>,
    pub(super) level_tiles: HashMap<(u32, u32), LevelTileType>,
    pub(super) level_tiles_flip: HashMap<(u32, u32), TileFlip>,
}

impl Level {
    fn ensure_unique_tileset(layer: tiled::FiniteTileLayer) -> Result<usize, anyhow::Error> {
        let mut result = None;

        for x in 0..layer.map().width {
            for y in 0..layer.map().height {
                if let Some(tile) = layer.get_tile(x as i32, y as i32) {
                    if result != Some(tile.tileset_index()) && result.replace(tile.tileset_index()).is_some() {
                        bail!("Found more than one tileset for the layer");
                    }
                }
            }
        }

        result.ok_or(anyhow!("The layer uses no tileset"))
    }

    pub fn new(
        tileset_indexing: Vec<TilesetIndexing>,
        map: &TiledMap, 
        tilesets: &LevelTilesetImages, 
        atlases: &Assets<TextureAtlas>,
    ) -> Result<Self, anyhow::Error> {
        // Get the level layer
        let level_layer = map.map.group_layer("level").ok_or_else(|| anyhow!("No `level` layer"))?;

        // Get geometry and activator layer
        let geometry_layer = level_layer.finite_tile_layer(GEOMETRY_LAYER_ID)
            .ok_or_else(|| anyhow!("No `{}` layer in `level`", GEOMETRY_LAYER_ID))?;
        let activator_layer = level_layer.finite_tile_layer(ACTIVATOR_LAYER_ID)
            .ok_or_else(|| anyhow!("No `{}` layer in `level`", ACTIVATOR_LAYER_ID))?;

        // Get the tilesets, ensuring that each layer uses exactly one.
        let geometry_tileset_id = Self::ensure_unique_tileset(geometry_layer)
            .context(format!("Failed to check that `{}` has one tileset", GEOMETRY_LAYER_ID))?;
        let activator_tileset_id = Self::ensure_unique_tileset(activator_layer)
            .context(format!("Failed to check that `{}` has one tileset", ACTIVATOR_LAYER_ID))?;

        // Scan the tilesets, creating mappings from tile IDs to engine data.
        let geometry_table = map.map.tilesets()[geometry_tileset_id].tile_properties()?;
        let activator_table = map.map.tilesets()[activator_tileset_id].tile_properties()?;

        let mut geometry_graphics = HashMap::new();
        let mut activators = HashMap::new();
        let mut level_tiles = HashMap::new();
        let mut level_tiles_flip = HashMap::new();

        for x in 0..map.map.width {
            for y in 0..map.map.height {
                let table_pos = (x, y);
                let y = (map.map.height - 1) as u32 - y;

                if let Some(act_tile) = activator_layer.get_tile_data(x as i32, y as i32) {
                    activators.insert(table_pos,
                        activator_table.get(&act_tile.id()).map(|tile: &ActivatorTile| tile.active)
                            .ok_or_else(|| anyhow!("No data for activator tile at ({x:}, {y:})"))?
                    );
                }
                
                if let Some(lvl_tile) = geometry_layer.get_tile_data(x as i32, y as i32) {
                    match geometry_table.get(&lvl_tile.id()) {
                        Some(GeometryTile::LevelTile(tile)) => { level_tiles.insert(table_pos, tile.ty); },
                        _ => bail!(anyhow!("No data for geometry tile at ({x:}, {y:})")),
                    }

                    level_tiles_flip.insert(table_pos, lvl_tile.bevy_flip_flags());
                    geometry_graphics.insert(table_pos, 
                        tileset_indexing[geometry_tileset_id].dispatch(lvl_tile.id())
                    );
                }
            }
        }

        Ok(Level {
            width: map.map.width,
            height: map.map.height,
            geometry_graphics,
            level_tiles,
            activators,
            level_tiles_flip,
            geometry_atlas: atlases.get(&tilesets.images[geometry_tileset_id])
                .unwrap().texture.clone(),
        })
    }

    pub fn width(&self) -> u32 { self.width }

    pub fn height(&self) -> u32 { self.height }
}
