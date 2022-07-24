use std::collections::HashMap;

use bevy_ecs_tilemap::prelude::*;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;

use super::cpu_tile_animation::Animations;
use super::tile_attributes;
use super::tile_animation;
use super::tile_commands;

use tile_commands::TileCommands;

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct Level {
    pub map: tiled::Map,
    pub tilesets: HashMap<usize, Handle<Image>>,
}

impl Level {
    fn spawn_layer_tiles(
        &self,
        tileset_index: usize,
        layer: tiled::Layer,
        commands: &mut Commands,
        builder: &mut LayerBuilder<TileBundle>,
        tile_init: &TileCommands,
    ) {
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

                            let entity = builder.get_tile_entity(commands, pos).unwrap();
                            tile_init[tileset_index as usize][&tile.id()](commands.entity(entity));

                            // NOTE who told me that this is correct? Nobody. I am going to
                            // research if it's possible to break this code or not
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
    }

    pub fn find_geometry_layer_id(&self) -> Option<u16> {
        self.map.layers().enumerate().find(|(_, x)| x.name == "geometry").map(|(x, _)| x as u16)
    }

    pub fn new(map: tiled::Map, tilesets: HashMap<usize, Handle<Image>>) -> Self {
        Level { map, tilesets }
    }

    // TODO consider shrinking the nested stuff
    pub fn spawn_map(&self, commands: &mut Commands, mut meshes: ResMut<Assets<Mesh>>, mut animations: ResMut<Animations>) -> (u16, Entity) {
        let map_id = 0;

        // Parse the map
        let attrs = tile_attributes::scan_tilesets(&self.map);
        let anims = tile_animation::scan_tilesets_for_animations(&self.map, &attrs);
        let mut anim_ids = vec![HashMap::new(); attrs.len()];
        let tile_init = tile_commands::build_commands(&attrs, &anims, &mut anim_ids, &mut *animations);

        // Create the loaded map
        let map_entity = commands.spawn().insert(Name::new("Map")).id();
        let mut map = Map::new(map_id, map_entity);

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
                    map_id,
                    layer_index as u16
                );

                self.spawn_layer_tiles(tileset_index, layer, commands, &mut builder, &tile_init);

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

        (map_id, map_entity)
    }
}
