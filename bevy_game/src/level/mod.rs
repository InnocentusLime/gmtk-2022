mod asset;
mod tile_attributes;
mod special_tiles;
mod tile_def;
mod tile_commands;
mod loader;
mod tile_animation;
mod cpu_tile_animation;

// TODO all tile_* files should be thrown into a `tile` submodule
use cpu_tile_animation::CPUTileAnimationPlugin;
use bevy_ecs_tilemap::prelude::*;
use bevy::prelude::*;

use loader::*;

pub use cpu_tile_animation::{ Animations, CPUAnimated };
pub use asset::{ Level, BaseLevelAssets };
pub use tile_def::{ ActivatableTileTag, MapReady, activeatable_tile_setup };
pub use special_tiles::StartTileTag;

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Level>()
            .add_event::<MapReady>()
            .add_asset_loader(TiledLoader)
            .add_plugin(TilemapPlugin)
            .add_system(add_set_texture_filter_to_nearest)
            .add_plugin(special_tiles::SpecialTilePlugin)
            .add_plugin(CPUTileAnimationPlugin);
    }
}

// TODO can we avoid doing that? What is the purpose of it
// anyway?
// Recommended by bevy_ecs_tilemap
fn add_set_texture_filter_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    use bevy::render::render_resource::TextureUsages;

    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.texture_descriptor.usage = 
                        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::COPY_DST;
                }
            }
            _ => (),
        }
    }
}

pub fn tile_pos_to_world_pos(
    tile_pos: (u32, u32),
    map_transform: &Transform,
    map: &mut MapQuery,
    map_id: u16,
    layer_id: u16,
) -> Vec2 {
    let layer = map.get_layer(map_id, layer_id).unwrap().1;
    let settings = &layer.settings;
    map_transform.mul_vec3(Vec3::new(
        tile_pos.0 as f32 * settings.tile_size.0 + settings.tile_size.0 / 2.0f32, 
        tile_pos.1 as f32 * settings.tile_size.1 + settings.tile_size.1 / 2.0f32, 
        0.0f32
    )).truncate()
}
