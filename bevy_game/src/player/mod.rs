pub mod dice;
mod systems;

use bevy_asset_loader::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::prelude::*;
use bevy::sprite::{ Mesh2dHandle, MaterialMesh2dBundle };
use iyes_loopless::prelude::*;

use crate::states::GameState;
use crate::level::tile_pos_to_world_pos;

pub use dice::Direction;

#[derive(StageLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct PlayerUpdateStage;

#[derive(SystemLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum PlayerSystems {
    Control,
    Update,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayerMoved>()
            .add_event::<PlayerChangingSide>()
            .add_event::<PlayerModification>()
            .add_stage_after(
                CoreStage::PreUpdate,
                PlayerUpdateStage,
                SystemStage::parallel()
            )
            .add_system_to_stage(
                PlayerUpdateStage,
                systems::player_controls.run_in_state(GameState::InGame).label(PlayerSystems::Control)
            )
            .add_system_to_stage(
                PlayerUpdateStage,
                systems::player_update.run_in_state(GameState::InGame).label(PlayerSystems::Update)
                    .after(PlayerSystems::Control)
            )
            .add_system_set_to_stage(
                PlayerUpdateStage,
                ConditionSet::new()
                    .after(PlayerSystems::Update).run_in_state(GameState::InGame)
                    .with_system(systems::player_animation)
                    .with_system(systems::player_camera)
                    .into()
            )
            /*.register_type::<PlayerState>()*/;
    }
}

#[derive(AssetCollection)]
pub struct BasePlayerAssets {
    #[asset(path = "models/Dice.glb")]
    pub player_gltf: Handle<bevy::gltf::Gltf>,
}

pub struct GeneratedPlayerAssets {
    pub model: Mesh2dHandle,
    pub material: Handle<ColorMaterial>,
}

impl FromWorld for GeneratedPlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<BasePlayerAssets>();
        let gltfs = world.resource::<Assets<bevy::gltf::Gltf>>();
        let gltf_meshes = world.resource::<Assets<bevy::gltf::GltfMesh>>();
        let std_materials = world.resource::<Assets<StandardMaterial>>();

        let player_gltf = gltfs.get(&assets.player_gltf).unwrap();
        let player_gltf = gltf_meshes.get(&player_gltf.meshes[0]).unwrap();
        let player_gltf = &player_gltf.primitives[0];
        let model = Mesh2dHandle(player_gltf.mesh.clone());

        let material = ColorMaterial {
            color: Color::WHITE,
            texture: std_materials.get(&player_gltf.material.as_ref().unwrap().to_owned())
                .unwrap().base_color_texture.to_owned(),
        };
        let material = world.resource_mut::<Assets<ColorMaterial>>().add(material);

        GeneratedPlayerAssets { material, model }
    }
}

pub fn spawn_player(
    commands: &mut Commands,
    map_id: u16,
    layer_id: u16,
    tile_pos: (u32, u32),
    map: &mut MapQuery,
    map_tf: &Transform,
    base_assets: &BasePlayerAssets,
    generated_assets: &GeneratedPlayerAssets,
) {
    let world_pos = tile_pos_to_world_pos(
        tile_pos,
        map_tf, 
        map, 
        map_id, 
        layer_id
    );
    commands.spawn()
        .insert(Name::new("Player"))
        .insert(PlayerState::AwaitingInput)
        .insert(Player::new(tile_pos, map_id, layer_id))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: generated_assets.model.clone(),
            material: generated_assets.material.clone(),
            // TODO hardcoded player size
            transform: Transform::from_translation(world_pos.extend(1.0f32))
                .with_scale(Vec3::new(25.0f32, 25.0f32, 25.0f32)),
            ..default()
        });
}

#[derive(Debug)]
pub enum PlayerModification {
    AcknowledgeMove,
    Roll(dice::Direction),
    Slide(dice::Direction),
    Kill,
    Escape,
}

pub struct PlayerMoved {
    pub cell: Entity,
    pub dice_state: dice::DiceEncoding,
}

pub struct PlayerChangingSide {
    pub dice_state: dice::DiceEncoding,
}

#[derive(Debug, Clone)]
pub enum MoveInfo {
    Slide,
    Rotate {
        start_rot: Quat,
        direction: dice::Direction,
    },
}

#[derive(Debug, Clone, Component)]
pub enum PlayerState {
    AwaitingInput,
    Moving {
        to: (u32, u32),
        to_entity: Entity,
        start_pos: Vec3,
        end_pos: Vec3,
        timer: Timer,
        info: MoveInfo,
    },
    AwaitingAcknowledge {
        new_pos: Vec3,
        new_rot: Quat,
    },
    Escaping {
        timer: Timer,
    },
}

#[derive(Clone, Component)]
pub struct Player {
    picked_anim: u8,
    map_id: u16, // The map ID the player traverses
    layer_id: u16, // The layer ID 
    // TODO state -> dice_state
    state: dice::DiceEncoding,
    current_cell: (u32, u32),
}

impl Player {
    pub fn new(start: (u32, u32), map_id: u16, layer_id: u16) -> Self {
        Player {
            picked_anim: 0,
            map_id,
            layer_id,
            current_cell: start,
            state: dice::DiceEncoding::new(),
        }
    }

    pub fn apply_rotation(&mut self, dir: dice::Direction) -> dice::DiceEncoding { self.state.apply_rotation(dir) }

    pub fn upper_side(&self) -> u8 { self.state.upper_side() }

    pub fn next_cell(&self, off: (i32, i32)) -> (u32, u32) {
        // FIXME MIGHT CRASH
        (
            (self.current_cell.0 as i32 + off.0) as u32,
            (self.current_cell.1 as i32 + off.1) as u32
        )
    }
}
