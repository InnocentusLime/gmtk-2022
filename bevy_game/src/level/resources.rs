use bevy_asset_loader::asset_collection::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use std::collections::HashMap;
use thiserror::Error;

use super::{ TiledMap, TilesetIndexing };
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Error, Debug)]
pub enum SublayerError {
    #[error("failed to find the sublayer")]
    NotFound,
    #[error("found duplicate sublayer definition")]
    Duplicate,
    #[error("the sublayer had a wrong type")]
    WrongType,
    #[error("the sublayer uses more than 1 tileset")]
    ManyTilesets,
    #[error("the sublayer uses no tileset")]
    NoTileset,
}

#[derive(Error, Debug)]
pub enum ActivatorTilesetError {
    #[error("tile with id {tile_id:} has no activation info")]
    TileWithoutInfo { tile_id: u32 },
    #[error("tile with id {tile_id:} contains a syntax error in its activation condition")]
    ConditionSyntaxError { tile_id: u32 },
}

#[derive(Error, Debug)]
pub enum GeometryTilesetError {
    #[error("tile with id {tile_id:} is declared with an unknown type: {ty:?}")]
    UnknownType { tile_id: u32, ty: String },
}

#[derive(Error, Debug)]
pub enum LevelInitError {
    #[error("the level layer is not a group layer")]
    LevelLayerNotGroupLayer,
    #[error("duplicate definition of the level layer")]
    LevelLayerDuplicate,
    #[error("no level layer found")]
    NoLevelLayer,
    #[error("encountered a problem with sublayer \"{name:}\"")]
    SublayerError {
        name: &'static str,
        source: SublayerError,
    },
    #[error("failed to parse activator tileset")]
    ActivatorTilesetError(#[from] ActivatorTilesetError),
    #[error("failed to parse geometry tileset")]
    GeometryTilesetError(#[from] GeometryTilesetError),
    #[error("incorrect geometry tile at ({x:}, {y:})")]
    IncorrectGeometryTile { x: u32, y: u32 },
    #[error("incorrect activator tile at ({x:}, {y:})")]
    IncorrectActivatorTile { x: u32, y: u32 },
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
    fn find_level_layer(map: &TiledMap) -> Result<tiled::GroupLayer, LevelInitError> {
        let mut level_layer = None;

        for layer in map.map.layers() {
            match layer.name.as_str() {
                "level" => match layer.layer_type() {
                    tiled::LayerType::GroupLayer(gr) => if level_layer.replace(gr).is_some() { 
                        return Err(LevelInitError::LevelLayerDuplicate)
                    },
                    _ => return Err(LevelInitError::LevelLayerNotGroupLayer),
                },
                _ => warn!("Layer \"{}\" will be ignored", layer.name),
            }
        }

        level_layer.ok_or(LevelInitError::NoLevelLayer)
    }

    fn find_finite_tile_sublayer<'a>(level_layer: tiled::GroupLayer<'a>, id: &'static str) -> Result<tiled::FiniteTileLayer<'a>, SublayerError> {
        let mut result = None;

        for layer in level_layer.layers() {
            if layer.name.as_str() == id {
                match layer.layer_type() {
                    tiled::LayerType::TileLayer(tiled::TileLayer::Finite(tl)) => if result.replace(tl).is_some() {
                        return Err(SublayerError::Duplicate)
                    },
                    _ => return Err(SublayerError::WrongType),
                }               
            }
        }

        result.ok_or(SublayerError::NotFound)
    }

    fn ensure_unique_tileset(layer: tiled::FiniteTileLayer, name: &'static str) -> Result<usize, SublayerError> {
        let mut result = None;

        for x in 0..layer.map().width {
            for y in 0..layer.map().height {
                if let Some(tile) = layer.get_tile(x as i32, y as i32) {
                    if result != Some(tile.tileset_index()) && result.replace(tile.tileset_index()).is_some() {
                        return Err(SublayerError::ManyTilesets);
                    }
                }
            }
        }

        result.ok_or(SublayerError::NoTileset)
    }

    fn scan_activator_tileset(activator_tileset: &tiled::Tileset) -> Result<HashMap<u32, ActivationCondition>, ActivatorTilesetError> {
        let mut result = HashMap::new();

        for (tile_id, tile) in activator_tileset.tiles() {
            match tile.properties.get("active") {
                None => return Err(ActivatorTilesetError::TileWithoutInfo { tile_id }),
                Some(tiled::PropertyValue::StringValue(x)) => match x.as_str() {
                    "odd" => { result.insert(tile_id, ActivationCondition::Odd); },
                    "even" => { result.insert(tile_id, ActivationCondition::Even); },
                    _ => return Err(ActivatorTilesetError::ConditionSyntaxError { tile_id }), 
                },
                Some(_) => return Err(ActivatorTilesetError::ConditionSyntaxError { tile_id }),
            }
        }

        Ok(result)
    }

    fn scan_geometry_tileset(geometry_tileset: &tiled::Tileset) -> Result<HashMap<u32, LevelTileType>, GeometryTilesetError> {
        let mut result = HashMap::new();

        for (tile_id, tile) in geometry_tileset.tiles() {
            match tile.tile_type.as_ref().map(String::as_str) {
                None => (),
                Some("conveyor") => { result.insert(tile_id, LevelTileType::Conveyor); },
                Some("fry") => { result.insert(tile_id, LevelTileType::Fry); },
                Some("floor") => { result.insert(tile_id, LevelTileType::Floor); },
                Some("player_start") => { result.insert(tile_id, LevelTileType::PlayerStart); },
                Some(ty) => return Err(GeometryTilesetError::UnknownType { tile_id, ty: ty.to_owned() }),
            }
        }

        Ok(result)
    }

    pub fn new(
        tileset_indexing: Vec<TilesetIndexing>,
        map: &TiledMap, 
        tilesets: &LevelTilesetImages, 
        atlases: &Assets<TextureAtlas>,
    ) -> Result<Self, LevelInitError> {
        // Get the level layer
        let level_layer = Self::find_level_layer(map)?;

        // Get geometry and activator layer
        let geometry_layer = Self::find_finite_tile_sublayer(level_layer, GEOMETRY_LAYER_ID)
            .map_err(|source| LevelInitError::SublayerError { source, name: GEOMETRY_LAYER_ID })?;
        let activator_layer = Self::find_finite_tile_sublayer(level_layer, ACTIVATOR_LAYER_ID)
            .map_err(|source| LevelInitError::SublayerError { source, name: ACTIVATOR_LAYER_ID })?;

        // Get the tilesets, ensuring that each layer uses exactly one.
        let geometry_tileset_id = Self::ensure_unique_tileset(geometry_layer, GEOMETRY_LAYER_ID)
            .map_err(|source| LevelInitError::SublayerError { source, name: GEOMETRY_LAYER_ID })?;
        let activator_tileset_id = Self::ensure_unique_tileset(activator_layer, ACTIVATOR_LAYER_ID)
            .map_err(|source| LevelInitError::SublayerError { source, name: ACTIVATOR_LAYER_ID })?;

        // Scan the tilesets, creating mappings from tile IDs to engine data.
        let geometry_table = Self::scan_geometry_tileset(&*map.map.tilesets()[geometry_tileset_id])?;
        let activator_table = Self::scan_activator_tileset(&*map.map.tilesets()[activator_tileset_id])?;

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
                        activator_table.get(&act_tile.id()).map(|t| *t)
                            .ok_or(LevelInitError::IncorrectGeometryTile { x, y })?
                    );
                }
                
                if let Some(lvl_tile) = geometry_layer.get_tile_data(x as i32, y as i32) {
                    level_tiles.insert(table_pos,
                        *geometry_table.get(&lvl_tile.id())
                            .ok_or(LevelInitError::IncorrectGeometryTile { x, y })?
                    );

                    level_tiles_flip.insert(table_pos, TileFlip {
                        x: lvl_tile.flip_h,
                        y: lvl_tile.flip_v,
                        d: lvl_tile.flip_d,
                    });

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
