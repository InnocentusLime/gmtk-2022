use std::collections::HashMap;
use std::io::BufReader;

use bevy::asset::{ AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset };
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_ecs_tilemap::prelude::*;

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<TiledMap>()
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

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct TiledMap {
    pub map: tiled::Map,
    pub tilesets: HashMap<usize, Handle<Image>>,
}

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
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
            info!("Loading a map!");
            let mut loader = tiled::Loader::new();
            let map = loader.load_tmx_map_from(BufReader::new(bytes), load_context.path())?;

            let mut dependencies = Vec::new();
            let mut handles = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                let tile_path = tileset.image.as_ref().unwrap().source.clone();
                let asset_path = AssetPath::new(tile_path, None);
                let texture: Handle<Image> = load_context.get_handle(asset_path.clone());

                handles.insert(tileset_index, texture.clone());

                dependencies.push(asset_path);
            }

            let loaded_asset = LoadedAsset::new(TiledMap {
                map, 
                tilesets: handles,
            }).with_dependencies(dependencies);

            load_context.set_default_asset(loaded_asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] { &["tmx"] }
}

pub struct LevelInfo {
    map_entity: Entity,
    pub start_tile: (u32, u32),
}

impl LevelInfo {
    fn spawn_map(
        commands: &mut Commands, 
        mut meshes: ResMut<Assets<Mesh>>,
        tiled_map: &TiledMap
    ) -> Entity {
        // Create the loaded map
        let map_entity = commands.spawn().insert(Name::new("Map")).id();
        let mut map = Map::new(0, map_entity);

        // Account for each tileset
        for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
            // Loop through each layer
            for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                let tile_width = tileset.tile_width as f32;
                let tile_height = tileset.tile_height as f32;

                let offset_x = layer.offset_x;
                let offset_y = layer.offset_y;

                let mut map_settings = LayerSettings::new(
                    MapSize(
                        (tiled_map.map.width as f32 / 64.0).ceil() as u32,
                        (tiled_map.map.height as f32 / 64.0).ceil() as u32,
                    ),
                    ChunkSize(64, 64),
                    TileSize(tile_width, tile_height),
                    TextureSize(
                        tileset.image.as_ref().unwrap().width as f32,
                        tileset.image.as_ref().unwrap().height as f32,
                    ),
                );

                map_settings.grid_size = Vec2::new(
                    tiled_map.map.tile_width as f32,
                    tiled_map.map.tile_height as f32,
                );

                map_settings.mesh_type = match tiled_map.map.orientation {
                    tiled::Orientation::Orthogonal => TilemapMeshType::Square,
                    _ => panic!("Bad tile format"),
                };

                let layer_entity = LayerBuilder::<TileBundle>::new_batch(
                    commands,
                    map_settings.clone(),
                    &mut meshes,
                    tiled_map
                        .tilesets
                        .get(&tileset_index)
                        .unwrap()
                        .clone_weak(),
                    0u16,
                    layer_index as u16,
                    move |mut tile_pos| {
                        if tile_pos.0 >= tiled_map.map.width
                            || tile_pos.1 >= tiled_map.map.height
                        {
                            return None;
                        }

                        if tiled_map.map.orientation == tiled::Orientation::Orthogonal {
                            tile_pos.1 = (tiled_map.map.height - 1) as u32 - tile_pos.1;
                        }

                        let x = tile_pos.0 as i32;
                        let y = tile_pos.1 as i32;

                        match layer.layer_type() {
                            tiled::LayerType::TileLayer(tile_layer) => {
                                tile_layer.get_tile(x, y).and_then(|tile| {
                                    // Skip tiles which don't use the tileset we 
                                    // are considering on the current iteration
                                    if tile.tileset_index() != tileset_index {
                                        return None;
                                    }
                                    let tile = Tile {
                                        texture_index: tile.id() as u16,
                                        flip_x: tile.flip_h,
                                        flip_y: tile.flip_v,
                                        flip_d: tile.flip_d,
                                        ..Default::default()
                                    };

                                    Some(TileBundle {
                                        tile,
                                        ..Default::default()
                                    })
                                })
                            },
                            _ => panic!("Unsupported layer type"),
                        }
                    },
                );

                commands.entity(layer_entity).insert(Transform::from_xyz(
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

    pub fn new(
        commands: &mut Commands, 
        meshes: ResMut<Assets<Mesh>>,
        tiled_map: &TiledMap
    ) -> Self {
        let map_entity = Self::spawn_map(commands, meshes, tiled_map);

        // TODO fix that ugly
        let start_tile_id = tiled_map.map.tilesets()[0].tiles()
            .find(|(_, x)| x.tile_type.as_ref().map(|ty| ty == "player_start").unwrap_or(false))
            .map(|x| x.0).expect("No tile of \"player_start\" type found");

        let geometry_layer = tiled_map.map.layers().find(|x| x.name == "geometry")
            .expect("No layer named \"geometry\" found");

        let tile_table = 
            match geometry_layer.layer_type() {
                tiled::LayerType::TileLayer(x) => x,
                _ => panic!("Geometry layer must have \"tile layer\" type"),
            };

        let mut start_tile = None;
        for x in 0..tiled_map.map.width {
            for y in 0..tiled_map.map.height {
                // TODO check tileset
                if let Some(tile) = tile_table.get_tile(x as i32, y as i32) {
                    if tile.id() == start_tile_id {
                        if start_tile.replace((x, y)).is_some() {
                            panic!("Multiple starts discovered");
                        }
                    }    
                }
            }
        }

        LevelInfo {
            map_entity,
            start_tile: start_tile.expect("No player start found"),
        }
    }

    pub fn map_entitiy(&self) -> Entity { self.map_entity }
}

// TODO accept layer ID
pub fn tile_pos_to_world_pos(
    mut tile_pos: (u32, u32),
    tiled_map: &TiledMap,
    map_transform: &Transform
) -> Vec3 { 
    // TODO remove linear search
    let geometry_layer = tiled_map.map.layers().find(|x| x.name == "geometry")
        .expect("No layer named \"geometry\" found");

    tile_pos.1 = (tiled_map.map.height - 1) as u32 - tile_pos.1;
    let offset_x = geometry_layer.offset_x * (tiled_map.map.tile_width as f32);
    let offset_y = geometry_layer.offset_y * (tiled_map.map.tile_height as f32);
    let (x, y) = (
        -offset_x + (tiled_map.map.tile_width * tile_pos.0) as f32, 
        offset_y + (tiled_map.map.tile_height * tile_pos.1) as f32
    );
    map_transform.mul_vec3(Vec3::new(x, y, 0.0f32))
}
