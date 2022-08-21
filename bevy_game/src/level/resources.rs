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

#[derive(Deserialize)]
pub enum GeometryTile {
    LogicTile {
        ty: LevelTileType
    },
    Frame,
    LevelTileAnimation {
        anim_ty: TileAnimationType,
        target: LevelTileType,
    },
}

#[derive(Deserialize)]
pub struct ActivatorTile {
    active: ActivationCondition,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum TileAnimationType {
    OnOffAnimation,
    OnAnimation,
    OffAnimation,
    OnTransitionAnimation,
    OffTransitionAnimation,
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

#[derive(Clone, Copy, Debug)]
pub struct LevelTile {
    pub texture: TileTexture,
    pub flip: TileFlip,
    pub ty: LevelTileType,
    pub activation_cond: Option<ActivationCondition>,
}

pub struct Level {
    width: u32,
    height: u32,
    pub(super) tiles: HashMap<(u32, u32), LevelTile>,
    pub(super) geometry_atlas: Handle<Image>,
}

impl Level {
    pub fn new(
        map: &TiledMap,
        tileset_indexing: &[TilesetIndexing],
        tilesets: &[&TextureAtlas],
    ) -> Result<Self, anyhow::Error> {
        static GEOMETRY_LAYER_ID: &'static str = "geometry";
        static ACTIVATOR_LAYER_ID: &'static str = "activators";

        // Get the level layer
        let level_layer = map.map.group_layer("level").ok_or_else(|| anyhow!("No `level` layer"))?;

        // Get geometry and activator layer
        let geometry_layer = level_layer.finite_tile_layer(GEOMETRY_LAYER_ID)
            .ok_or_else(|| anyhow!("No `{}` layer in `level`", GEOMETRY_LAYER_ID))?;
        let activator_layer = level_layer.finite_tile_layer(ACTIVATOR_LAYER_ID)
            .ok_or_else(|| anyhow!("No `{}` layer in `level`", ACTIVATOR_LAYER_ID))?;

        // Get the tilesets, ensuring that each layer uses exactly one.
        let geometry_tileset_id = ensure_unique_tileset(geometry_layer)
            .context(format!("Failed to check that `{}` has one tileset", GEOMETRY_LAYER_ID))?;
        let activator_tileset_id = ensure_unique_tileset(activator_layer)
            .context(format!("Failed to check that `{}` has one tileset", ACTIVATOR_LAYER_ID))?;

        // Deserialize tile properties
        let geometry_table = map.map.tilesets()[geometry_tileset_id].tile_properties()?;
        let activator_table: HashMap<u32, ActivatorTile> = map.map.tilesets()[activator_tileset_id].tile_properties()?;

        // Build level tiles
        let mut tiles = HashMap::new();

        for x in 0..map.map.width {
            for y in 0..map.map.height {
                let table_pos = (x, y);
                let y = (map.map.height - 1) as u32 - y;

                let logic_tile = match geometry_layer.get_tile_data(x as i32, y as i32) {
                    Some(x) => x,
                    None => continue,
                };

                let ty = match geometry_table.get(&logic_tile.id()) {
                    Some(GeometryTile::LogicTile { ty }) => *ty,
                    _ => bail!("Incorrect tile on logic layer at ({x:}, {y:}) (not a logic tile)"),
                };

                let activation_cond = 
                    activator_layer.get_tile_data(x as i32, y as i32)
                        .map(|tile| 
                            activator_table.get(&tile.id()).map(|t| t.active)
                            .ok_or_else(|| anyhow!("Tile at ({x:}, {y:}) on activator layer isn't an activator tile"))
                        )
                        .transpose()?;

                tiles.insert(table_pos, LevelTile {
                    ty,
                    activation_cond,
                    flip: logic_tile.bevy_flip_flags(),
                    texture: TileTexture(
                        tileset_indexing[geometry_tileset_id].dispatch(logic_tile.id())
                    ),
                });
            }
        }

        Ok(Level {
            width: map.map.width,
            height: map.map.height,
            tiles,
            geometry_atlas: tilesets[geometry_tileset_id].texture.clone(),
        })
    }

    pub fn width(&self) -> u32 { self.width }

    pub fn height(&self) -> u32 { self.height }
}
