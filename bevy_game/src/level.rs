use std::collections::HashMap;
use std::io::BufReader;

use bevy::asset::{ AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset };
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_ecs_tilemap::prelude::*;

use crate::special_tiles::*;

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Level>()
            .add_asset_loader(TiledLoader)
            .add_plugin(TilemapPlugin)
            .add_system(add_set_texture_filter_to_nearest);
    }
}

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
    layer_id: u32,
) -> Vec2 {
    let layer = map.get_layer(map_id as u16, layer_id as u16).unwrap().1;
    let settings = &layer.settings;
    map_transform.mul_vec3(Vec3::new(
        tile_pos.0 as f32 * settings.tile_size.0 + settings.tile_size.0 / 2.0f32, 
        tile_pos.1 as f32 * settings.tile_size.1 + settings.tile_size.1 / 2.0f32, 
        0.0f32
    )).truncate()
}

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct Level {
    pub map: tiled::Map,
    pub tilesets: HashMap<usize, Handle<Image>>,
}

impl Level {
    fn find_geometry_layer_id(&self) -> Option<usize> {
        self.map.layers().find(|x| x.name == "geometry")
            .map(|x| x.id() as usize)
    }
    
    pub fn new(
        map: tiled::Map,
        tilesets: HashMap<usize, Handle<Image>>,
    ) -> Self {
        Level {
            map,
            tilesets,
        }
    }

    pub fn spawn_map(
        &self,
        commands: &mut Commands, 
        mut meshes: ResMut<Assets<Mesh>>,
    ) -> Entity {
        // Create the loaded map
        let map_entity = commands.spawn().insert(Name::new("Map")).id();
        let mut map = Map::new(0, map_entity);

        // Account for each tileset
        for (tileset_index, tileset) in self.map.tilesets().iter().enumerate() {
            // Loop through each layer
            for (layer_index, layer) in self.map.layers().enumerate() {
                let tile_width = tileset.tile_width as f32;
                let tile_height = tileset.tile_height as f32;

                let offset_x = layer.offset_x;
                let offset_y = layer.offset_y;

                let mut map_settings = LayerSettings::new(
                    MapSize(
                        (self.map.width as f32 / 64.0).ceil() as u32,
                        (self.map.height as f32 / 64.0).ceil() as u32,
                    ),
                    ChunkSize(64, 64),
                    TileSize(tile_width, tile_height),
                    TextureSize(
                        tileset.image.as_ref().unwrap().width as f32,
                        tileset.image.as_ref().unwrap().height as f32,
                    ),
                );

                map_settings.grid_size = Vec2::new(
                    self.map.tile_width as f32,
                    self.map.tile_height as f32,
                );

                map_settings.mesh_type = match self.map.orientation {
                    tiled::Orientation::Orthogonal => TilemapMeshType::Square,
                    _ => panic!("Bad tile format"),
                };

                let (mut builder, layer_entity) = LayerBuilder::<TileBundle>::new(
                    commands,
                    map_settings.clone(),
                    0u16,
                    layer_index as u16
                );

                for x in 0..self.map.width {
                    for y in 0..self.map.height {
                        match layer.layer_type() {
                            tiled::LayerType::TileLayer(tile_layer) => {
                                tile_layer.get_tile(x as i32, y as i32).map(|tile| {
                                    let y = (self.map.height - 1) as u32 - y;
                                    let pos = TilePos(x, y);

                                    // Skip tiles which don't use the tileset we 
                                    // are considering on the current iteration
                                    if tile.tileset_index() != tileset_index {
                                        return;
                                    }

                                    if let Some(tile_meta) = tile.get_tile() {
                                        let entity = builder.get_tile_entity(commands, pos).unwrap();
                                        match tile_meta.tile_type.as_ref().map(|x| x.as_str()) {
                                            Some("player_start") => {
                                                commands.entity(entity).insert(StartTileTag)
                                                    .insert(SolidTileTag);
                                            },
                                            Some("player_end") => {
                                                commands.entity(entity).insert(EndTileTag);
                                            },
                                            _ => {
                                                commands.entity(entity).insert(SolidTileTag);
                                            },
                                        }
                                    }

                                    let tile = 
                                        Tile {
                                            texture_index: tile.id() as u16,
                                            flip_x: tile.flip_h,
                                            flip_y: tile.flip_v,
                                            flip_d: tile.flip_d,
                                            ..Default::default()
                                        };

                                    builder.set_tile(pos, TileBundle { tile, ..default() }).unwrap();
                                });
                            },
                            _ => panic!("Unsupported layer type"),
                        }
                    }
                }

                let layer_bundle = 
                    builder.build(
                       commands,
                        &mut meshes,
                        self
                        .tilesets
                        .get(&tileset_index)
                        .unwrap()
                        .clone_weak()
                    );

                commands.entity(layer_entity)
                .insert_bundle(layer_bundle).insert(Transform::from_xyz(
                    offset_x,
                    -offset_y,
                    layer_index as f32,
                ));
                map.add_layer(commands, layer_index as u16, layer_entity);
            }
        }

        commands.entity(map_entity).insert(map);

        map_entity
    }
}

#[derive(Clone, Copy, Default)]
pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            use std::path::Path; 

            let mut loader = tiled::Loader::new();
            // TODO not entirely correct
            let map = loader.load_tmx_map_from(BufReader::new(bytes), &Path::new("assets/").join(load_context.path()))?;

            let mut dependencies = Vec::new();
            let mut handles = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                // TODO mhm
                let tile_path = tileset.image.as_ref().unwrap().source.strip_prefix("assets/").unwrap();
                let asset_path = AssetPath::new(tile_path.to_path_buf(), None);
                let texture: Handle<Image> = load_context.get_handle(asset_path.clone());

                handles.insert(tileset_index, texture.clone());

                dependencies.push(asset_path);
            }

            let loaded_asset = LoadedAsset::new(Level::new(
                map, handles
            )).with_dependencies(dependencies);

            load_context.set_default_asset(loaded_asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] { &["tmx"] }
}
