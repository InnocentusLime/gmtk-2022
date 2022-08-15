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
use crate::level::{ LevelTag, tile_pos_to_world_pos };

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
    start_q: Query<&TilePos, With<StartTileTag>>,
    map_q: Query<(&Transform, &TilemapGridSize), With<LevelTag>>,
    generated_assets: Res<GeneratedPlayerAssets>,
) {
    let start_pos = start_q.single();
    let (map_tf, map_grid) = map_q.single();
    let start_world_pos = tile_pos_to_world_pos(*start_pos, map_tf, map_grid);

    commands.spawn()
        .insert(PlayerTag)
        .insert(Name::new("Player"))
        .insert(Moveable::new(*start_pos))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: generated_assets.model.clone(),
            material: generated_assets.material.clone(),
            // TODO hardcoded player size
            // FIXME feels weird to double-set player's pos
            transform: Transform::from_translation(start_world_pos.extend(1.0f32))
                .with_scale(Vec3::new(25.0f32, 25.0f32, 25.0f32)),
            ..default()
        });
}
