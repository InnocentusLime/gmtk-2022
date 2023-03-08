mod resources;

use std::{time::Duration, path::Path};
use std::collections::HashMap;

use bevy::asset::AssetPath;
use bevy_ecs_tilemap::prelude::*;
use bevy::{prelude::*};

use bevy_ecs_tilemap_cpu_anim::{CPUTileAnimation, Frame};
pub use resources::*;

use crate::tile::*;
use serde::Deserialize;
use bevy_tiled::*;
use crate::moveable::MoveableTilemapTag;

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(TiledPlugin)
            .add_plugin(TilemapPlugin)
            .add_plugin(TilePlugin);
    }
}

pub fn tile_pos_to_world_pos(
    tile_pos: TilePos,
    map_transform: &Transform,
    map_grid: &TilemapGridSize,
) -> Vec2 {
    map_transform.transform_point(
        tile_pos.center_in_world(
            map_grid,
            &TilemapType::Square,
        ).extend(0.0f32)
    ).truncate()
}

pub fn get_level_map(
    base_level_assets: Res<BaseLevelAssets>
) -> Handle<TiledMap> {
    base_level_assets.map.clone()
}

#[derive(Default, Clone, Copy, Debug, Deserialize)]
struct TileAnimationFrame {
    id: u32,
    dur: u64,
}

#[derive(Default, Debug, Deserialize)]
struct TileAnimation(Vec<TileAnimationFrame>);

impl TileAnimation {
    fn decode(self, indexing: &TilesetIndexing) -> CPUTileAnimation {
        CPUTileAnimation::new(
            self.0.into_iter()
                .map(|frame| Frame {
                    texture_id: indexing.dispatch(frame.id),
                    duration: Duration::from_millis(frame.dur),
                })
        )
    }
}

impl ActivatableAnimating<TileAnimation> {
    fn decode(
        self,
        tileset: usize,
        tile: u32,
        assets: &mut Assets<CPUTileAnimation>,
        map_path: Option<&Path>,
        indexing: &TilesetIndexing,
    ) -> ActivatableAnimating {
        let mut acquire_asset = |anim, tag: &'static str| {
            let asset = TileAnimation::decode(anim, indexing);
            debug!("anim{tileset:}_{tile:}_{tag:}\n{asset:?}");

            match map_path {
                Some(path) => assets.set(AssetPath::new_ref(
                    path, Some(&format!("anim{tileset:}_{tile:}_{tag:}"))
                ), asset),
                None => assets.add(asset),
            }
        };

        match self {
            ActivatableAnimating::None => ActivatableAnimating::None,
            ActivatableAnimating::Pause { on_anim } => ActivatableAnimating::Pause {
                on_anim: acquire_asset(on_anim, "on_anim"),
            },
            ActivatableAnimating::Switch {
                on_transit: on_transition,
                off_transit: off_transition,
                on_anim,
                off_anim,
            } => ActivatableAnimating::Switch {
                on_transit: acquire_asset(on_transition, "on_transition"),
                off_transit: acquire_asset(off_transition, "off_transition"),
                on_anim: acquire_asset(on_anim, "off_anim"),
                off_anim: acquire_asset(off_anim, "on_anim"),
            }
        }
    }
}

struct GraphicsTileBuilder<'a> {
    map_path: Option<&'a Path>,
    anims: &'a mut Assets<CPUTileAnimation>,
    deserialized_props: HashMap<(usize, u32), GraphicsTileBundle>,
}

impl<'a> TileBuilder for GraphicsTileBuilder<'a> {
    fn process_tileset(
        &mut self,
        set_id: usize,
        tileset: &tiled::Tileset,
        indexing: &TilesetIndexing,
    ) -> anyhow::Result<()> {
        self.deserialized_props.reserve(tileset.tilecount as usize);

        for (id, tile) in tileset.tiles() {
            let props: GraphicsTileBundle<TileAnimation> = tile.properties()?;
            self.deserialized_props.insert(
                (set_id, id),
                GraphicsTileBundle {
                    animating: props.animating.decode(
                        set_id,
                        id,
                        self.anims,
                        self.map_path,
                        indexing
                    ),
                }
            );
        }

        Ok(())
    }

    fn build(
        &mut self,
        set_id: usize,
        id: u32,
        cmds: &mut bevy::ecs::system::EntityCommands,
    ) -> anyhow::Result<()> {
        cmds.insert(self.deserialized_props[&(set_id, id)].clone());

        Ok(())
    }

    fn finish_layer(
        &mut self,
        _set_id: usize,
        cmds: &mut bevy::ecs::system::EntityCommands,
    ) -> anyhow::Result<()> {
        cmds.insert(GraphicsTilemapTag);

        Ok(())
    }
}

// NOTE I don't think I can do anything here to satisfy clippy.
// Maybe some further investigation will prove me wrong.
#[allow(clippy::too_many_arguments)]
pub fn spawn_level(
    In(tilemap_texture_data): In<Vec<(TilesetIndexing, TilemapTexture)>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    base_level_assets: Res<BaseLevelAssets>,
    maps: Res<Assets<TiledMap>>,
    mut animations: ResMut<Assets<CPUTileAnimation>>,
) {
    let map_asset_path = asset_server.get_handle_path(base_level_assets.map.clone());

    let mut logic_tile_builder = BasicDeserBuilder::<LogicTileBundle, _>::new(|cmds| {
        cmds
            .insert((
                LogicTilemapTag,
                MoveableTilemapTag,
                Visibility { is_visible: false },
            ));
    });
    let mut trigger_tile_builder = BasicDeserBuilder::<TriggerTileBundle, _>::new(|cmds| {
        cmds
            .insert((
                TriggerTilemapTag,
                Visibility { is_visible: false },
            ));
    });
    let mut graphics_tile_builder = GraphicsTileBuilder {
        map_path: map_asset_path.as_ref().map(|x| x.path()),
        anims: &mut animations,
        deserialized_props: HashMap::new(),
    };

    let res = MapParser::new(
        &mut commands,
        SimpleCallbackSelector {
            pool: [
                &mut logic_tile_builder,
                &mut trigger_tile_builder,
                &mut graphics_tile_builder,
            ],
            picker: |name| match name {
                "logic_tiles" => 0,
                "activator_tiles" => 1,
                _ => 2,
            },
        },
        &tilemap_texture_data,
    )
    .parse_map(&maps.get(&base_level_assets.map).unwrap().map);

    if let Err(e) = res {
        error!("Error parsing map: {e}");

        let mut e = e.source();
        while let Some(rec) = e.and_then(std::error::Error::source) {
            e = rec.source();
            error!("{rec}");
        }
    }
}