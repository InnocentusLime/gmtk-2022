use anyhow::{ Context, anyhow, bail };
use bevy_asset_loader::asset_collection::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap_cpu_anim::*;
use bevy_tiled::*;
use bevy::prelude::*;
use std::{collections::HashMap, time::Duration};
use serde::{Deserialize, Deserializer};
use bevy_tiled::{ TiledMap, TilesetIndexing };

use crate::tile::{ ActivationCondition, ActivatableAnimating, TileKind };

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[repr(u8)]
pub enum TileAnimationType {
    OnOffAnimation,
    OnAnimation,
    OffAnimation,
    OnTransition,
    OffTransition,
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

    result.ok_or_else(|| anyhow!("The layer uses no tileset"))
}

#[derive(Deserialize)]
struct LogicTile {
    ty: TileKind,
}

#[derive(Deserialize)]
struct TriggerTile {
    active: ActivationCondition,
}

#[derive(Clone, Copy, Debug)]
pub struct LogicTileData {
    pub texture: TileTexture,
    pub flip: TileFlip,
    pub ty: TileKind,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct TileAnimationFrame {
    id: u32,
    dur: u64,
}

#[derive(Debug, Deserialize)]
struct TileAnimation(Vec<TileAnimationFrame>);

impl TileAnimation {
    fn into_cpu_tile_anim(self, mapping: &TilesetIndexing) -> CPUTileAnimation {
        CPUTileAnimation::from_frames(
            self.0.into_iter()
                .map(|frame| Frame {
                    texture_id: mapping.dispatch(frame.id),
                    duration: Duration::from_millis(frame.dur),
                })
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GraphicsTileAnimationType {
    None,
    Pause {
        on_anim: TileAnimation,
    },
    Switch {
        on_transition: TileAnimation,
        off_transition: TileAnimation,
        on_anim: TileAnimation,
        off_anim: TileAnimation,
    },
}

fn deserialize_anim_ty<'de, D>(des: D) -> Result<GraphicsTileAnimationType, D::Error>
where
    D: Deserializer<'de>
{
    let s = <&'_ str as Deserialize>::deserialize(des)?;
    Ok(
        serde_json::from_str(s)
        .map_err(|e| <D::Error as serde::de::Error>::custom(e))?
    )
}

#[derive(Deserialize)]
struct GraphicsTile {
    #[serde(deserialize_with = "deserialize_anim_ty")]
    animation_type: GraphicsTileAnimationType,
}

#[derive(Clone, Copy, Debug)]
pub struct GraphicsTileData {
    pub texture: TileTexture,
    pub flip: TileFlip,
    pub activatable_animating: Option<ActivatableAnimating>,
}

#[derive(Clone, Copy, Debug)]
pub struct TriggerTileData {
    pub texture: TileTexture,
    pub activation_cond: ActivationCondition,
}

pub struct Level {
    width: u32,
    height: u32,
    // Logic layer
    pub(super) logic_tiles: HashMap<(u32, u32), LogicTileData>,
    pub(super) logic_atlas: Handle<Image>,
    // Graphics layer
    pub(super) graphics_tiles: HashMap<(u32, u32), GraphicsTileData>,
    pub(super) graphics_atlas: Handle<Image>,
    // Trigger layer
    pub(super) trigger_tiles: HashMap<(u32, u32), TriggerTileData>,
    pub(super) trigger_atlas: Handle<Image>,
}

impl Level {
    pub fn new(
        map: &TiledMap,
        cpu_tile_animations: &mut CPUTileAnimations,
        tileset_indexing: &[TilesetIndexing],
        tilesets: &[&TextureAtlas],
    ) -> Result<Self, anyhow::Error> {
        static LOGIC_LAYER_ID: &str = "logic";
        static GRAPHICS_LAYER_ID: &str = "graphics";
        static TRIGGER_LAYER_ID: &str = "triggers";

        // Get the level layer
        let level_layer = map.map.group_layer("level").ok_or_else(|| anyhow!("No `level` layer"))?;

        // Logic tiles
        let (logic_tiles, logic_atlas) = {
            let logic_layer = level_layer.finite_tile_layer(LOGIC_LAYER_ID)
                .ok_or_else(|| anyhow!("No `{}` layer in `level`", LOGIC_LAYER_ID))?;
            let logic_tileset_id = ensure_unique_tileset(logic_layer)
                .context(format!("Failed to check that `{}` has one tileset", LOGIC_LAYER_ID))?;
            let logic_properties: HashMap<_, LogicTile> = 
                map.map.tilesets()[logic_tileset_id].tile_properties()?;
            let mut logic_tiles = HashMap::new();

            for x in 0..map.map.width {
                for y in 0..map.map.height {
                    let table_pos = (x, y);
                    let y = (map.map.height - 1) as u32 - y;
                    let logic_tile = match logic_layer.get_tile_data(x as i32, y as i32) {
                        Some(x) => x,
                        None => continue,
                    };

                    logic_tiles.insert(
                        table_pos, 
                        LogicTileData {
                            ty: logic_properties[&logic_tile.id()].ty,
                            flip: logic_tile.bevy_flip_flags(),
                            texture: TileTexture(
                                tileset_indexing[logic_tileset_id].dispatch(logic_tile.id())
                            ),
                        }
                    );
                }
            }

            (logic_tiles, tilesets[logic_tileset_id].texture.clone())
        };

        // Trigger tiles
        let (trigger_tiles, trigger_atlas) = {
            let trigger_layer = level_layer.finite_tile_layer(TRIGGER_LAYER_ID)
                .ok_or_else(|| anyhow!("No `{}` layer in `level`", TRIGGER_LAYER_ID))?;
            let trigger_tileset_id = ensure_unique_tileset(trigger_layer)
                .context(format!("Failed to check that `{}` has one tileset", TRIGGER_LAYER_ID))?;
            let trigger_properties: HashMap<_, TriggerTile> = 
                map.map.tilesets()[trigger_tileset_id].tile_properties()?;
            let mut trigger_tiles = HashMap::new();

            for x in 0..map.map.width {
                for y in 0..map.map.height {
                    let table_pos = (x, y);
                    let y = (map.map.height - 1) as u32 - y;
                    let trigger_tile = match trigger_layer.get_tile_data(x as i32, y as i32) {
                        Some(x) => x,
                        None => continue,
                    };

                    trigger_tiles.insert(
                        table_pos, 
                        TriggerTileData {
                            activation_cond: trigger_properties[&trigger_tile.id()].active,
                            texture: TileTexture(
                                tileset_indexing[trigger_tileset_id].dispatch(trigger_tile.id())
                            ),
                        }
                    );
                }
            }

            (trigger_tiles, tilesets[trigger_tileset_id].texture.clone())
        };

        // Graphics tiles
        let (graphics_tiles, graphics_atlas) = {
            let graphics_layer = map.map.finite_tile_layer(GRAPHICS_LAYER_ID)
                .ok_or_else(|| anyhow!("No `{}` layer", GRAPHICS_LAYER_ID))?;
            let graphics_tileset_id = ensure_unique_tileset(graphics_layer)
                .context(format!("Failed to check that `{}` has one tileset", GRAPHICS_LAYER_ID))?;
            let graphics_indexing = &tileset_indexing[graphics_tileset_id];
            let graphics_properties: HashMap<_, GraphicsTile> = 
                map.map.tilesets()[graphics_tileset_id].tile_properties()?;
            let graphics_anims: HashMap<_, ActivatableAnimating> = 
                graphics_properties.into_iter()
                .filter_map(|(tile, data)| match data.animation_type {
                    GraphicsTileAnimationType::None => None,
                    GraphicsTileAnimationType::Pause { on_anim } => Some((
                        tile, 
                        ActivatableAnimating::Pause { 
                            anim: cpu_tile_animations.add_animation(on_anim.into_cpu_tile_anim(graphics_indexing)), 
                        }
                    )),
                    GraphicsTileAnimationType::Switch { 
                        on_transition, off_transition, on_anim, off_anim 
                    } => Some ((
                        tile,
                        ActivatableAnimating::Switch { 
                            on_transition: cpu_tile_animations.add_animation(on_transition.into_cpu_tile_anim(graphics_indexing)), 
                            off_transition: cpu_tile_animations.add_animation(off_transition.into_cpu_tile_anim(graphics_indexing)), 
                            on_anim: cpu_tile_animations.add_animation(on_anim.into_cpu_tile_anim(graphics_indexing)), 
                            off_anim: cpu_tile_animations.add_animation(off_anim.into_cpu_tile_anim(graphics_indexing)),
                        }
                    )),
                })
                .collect();
            let mut graphics_tiles = HashMap::new();

            for x in 0..map.map.width {
                for y in 0..map.map.height {
                    let table_pos = (x, y);
                    let y = (map.map.height - 1) as u32 - y;
                    let graphics_tile = match graphics_layer.get_tile_data(x as i32, y as i32) {
                        Some(x) => x,
                        None => continue,
                    };

                    graphics_tiles.insert(
                        table_pos, 
                        GraphicsTileData { 
                            texture: TileTexture(
                                graphics_indexing.dispatch(graphics_tile.id())
                            ), 
                            flip: graphics_tile.bevy_flip_flags(), 
                            activatable_animating: graphics_anims.get(&graphics_tile.id()).map(|x| *x),
                        }
                    );
                }
            }

            (graphics_tiles, tilesets[graphics_tileset_id].texture.clone())
        };

        Ok(Level {
            width: map.map.width,
            height: map.map.height,
            // Logic layer
            logic_tiles,
            logic_atlas,
            // Trigger layer
            trigger_tiles,
            trigger_atlas,
            // Graphics layer
            graphics_tiles,
            graphics_atlas,
        })
    }

    pub fn width(&self) -> u32 { self.width }

    pub fn height(&self) -> u32 { self.height }
}
