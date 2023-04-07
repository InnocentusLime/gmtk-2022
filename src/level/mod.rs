mod resources;

use bevy_ecs_tilemap::prelude::*;
use bevy::prelude::*;

pub use resources::*;

use crate::tile::*;

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
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

// #[derive(Default, Clone, Copy, Debug, Deserialize)]
// struct TileAnimationFrame {
//     id: u32,
//     dur: u64,
// }

// #[derive(Default, Debug, Deserialize)]
// struct TileAnimation(Vec<TileAnimationFrame>);

// impl TileAnimation {
//     fn decode(self, indexing: &TilesetIndexing) -> CPUTileAnimation {
//         CPUTileAnimation::new(
//             self.0.into_iter()
//                 .map(|frame| Frame {
//                     texture_id: indexing.dispatch(frame.id),
//                     duration: Duration::from_millis(frame.dur),
//                 })
//         )
//     }
// }

// impl GraphicsAnimating<TileAnimation> {
//     fn decode(
//         self,
//         tileset: usize,
//         tile: u32,
//         assets: &mut Assets<CPUTileAnimation>,
//         map_path: Option<&Path>,
//         indexing: &TilesetIndexing,
//     ) -> GraphicsAnimating {
//         let mut acquire_asset = |anim, tag: &'static str| {
//             let asset = TileAnimation::decode(anim, indexing);
//             debug!("anim{tileset:}_{tile:}_{tag:}\n{asset:?}");

//             match map_path {
//                 Some(path) => assets.set(AssetPath::new_ref(
//                     path, Some(&format!("anim{tileset:}_{tile:}_{tag:}"))
//                 ), asset),
//                 None => assets.add(asset),
//             }
//         };

//         GraphicsAnimating {
//             on_transit: acquire_asset(self.on_transit, "on_transition"),
//             off_transit: acquire_asset(self.off_transit, "off_transition"),
//             on_anim: acquire_asset(self.on_anim, "off_anim"),
//             off_anim: acquire_asset(self.off_anim, "on_anim"),
//         }
//     }
// }

// struct GraphicsTileBuilder<'a> {
//     map_path: Option<&'a Path>,
//     anims: &'a mut Assets<CPUTileAnimation>,
//     deserialized_props: HashMap<(usize, u32), GraphicsTileBundle>,
// }

// impl<'a> TileBuilder for GraphicsTileBuilder<'a> {
//     fn process_tileset(
//         &mut self,
//         set_id: usize,
//         tileset: &tiled::Tileset,
//         indexing: &TilesetIndexing,
//     ) -> anyhow::Result<()> {
//         self.deserialized_props.reserve(tileset.tilecount as usize);

//         for (id, tile) in tileset.tiles() {
//             let props: GraphicsTileBundle<TileAnimation> = tile.properties()?;
//             self.deserialized_props.insert(
//                 (set_id, id),
//                 GraphicsTileBundle {
//                     animating: props.animating.decode(
//                         set_id,
//                         id,
//                         self.anims,
//                         self.map_path,
//                         indexing
//                     ),
//                 }
//             );
//         }

//         Ok(())
//     }

//     fn build(
//         &mut self,
//         set_id: usize,
//         id: u32,
//         cmds: &mut bevy::ecs::system::EntityCommands,
//     ) -> anyhow::Result<()> {
//         cmds.insert(self.deserialized_props[&(set_id, id)].clone());

//         Ok(())
//     }

//     fn finish_layer(
//         &mut self,
//         _set_id: usize,
//         cmds: &mut bevy::ecs::system::EntityCommands,
//     ) -> anyhow::Result<()> {
//         cmds.insert(GraphicsTilemapTag);

//         Ok(())
//     }
// }

// NOTE I don't think I can do anything here to satisfy clippy.
// Maybe some further investigation will prove me wrong.
pub fn spawn_level() {}