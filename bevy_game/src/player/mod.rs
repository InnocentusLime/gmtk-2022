mod dice;
mod events;
mod components;
mod resources;
mod systems;

use bevy_ecs_tilemap::prelude::*;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use iyes_loopless::prelude::*;

use crate::moveable::Moveable;
use crate::tile::StartTileTag;
use crate::states::GameState;
use crate::level::{ LevelInfo, tile_pos_to_world_pos };

pub use dice::*;
pub use resources::*;
pub use components::*;
pub use events::*;

use systems::*;

#[derive(StageLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct PlayerInputStage;

#[derive(StageLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct PlayerPostStage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayerEscapedEvent>()
            .add_event::<PlayerMoved>()
            .add_event::<PlayerChangingSide>()
            .add_stage_after(CoreStage::Update, PlayerInputStage, SystemStage::parallel())
            .add_stage_before(CoreStage::PostUpdate, PlayerPostStage, SystemStage::parallel())
            .add_system_to_stage(
                PlayerInputStage,
                player_controls.run_in_state(GameState::InGame)
            )
            .add_system_set_to_stage(
                PlayerPostStage,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(player_camera)
                    .with_system(player_win_anim)
                    .with_system(player_win_sound)
                    .into()
            );
    }
}

pub fn spawn_player(
    mut commands: Commands,
    mut map_q: MapQuery,
    start: Query<&TilePos, With<StartTileTag>>,
    map_entity: Query<(&LevelInfo, &Transform)>,
    generated_assets: Res<GeneratedPlayerAssets>,
) {
    let tile_pos = start.single();
    let tile_pos = (tile_pos.0, tile_pos.1);
    let (level_info, map_tf) = map_entity.single();
    let world_pos = tile_pos_to_world_pos(
        tile_pos,
        map_tf, 
        &mut map_q, 
        level_info.map(), 
        level_info.geometry_layer()
    );
    commands.spawn()
        .insert(PlayerTag)
        .insert(Name::new("Player"))
        .insert(Moveable::new(tile_pos))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: generated_assets.model.clone(),
            material: generated_assets.material.clone(),
            // TODO hardcoded player size
            transform: Transform::from_translation(world_pos.extend(1.0f32))
                .with_scale(Vec3::new(25.0f32, 25.0f32, 25.0f32)),
            ..default()
        });
}
