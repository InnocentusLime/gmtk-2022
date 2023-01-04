mod components;
mod events;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::level::tile_pos_to_world_pos;
use crate::moveable::{MoveableBundle, MoveableTilemapTag, self};
use crate::states::GameState;
use crate::tile::{TileKind, TileUpdateStage};

pub use components::*;
pub use events::*;
pub use resources::*;
pub use systems::*;

#[derive(StageLabel, Debug, PartialEq, Eq, Hash, Clone)]
pub struct PlayerInputStage;

#[derive(StageLabel, Debug, PartialEq, Eq, Hash, Clone)]
pub struct PlayerPostStage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerEscapedEvent>()
            .add_stage_after(
                TileUpdateStage,
                PlayerInputStage,
                SystemStage::parallel(),
            )
            .add_stage_before(
                CoreStage::PostUpdate,
                PlayerPostStage,
                SystemStage::parallel(),
            )
            .add_system_to_stage(
                PlayerInputStage,
                player_controls.run_in_state(GameState::InGame),
            )
            .add_system_set_to_stage(
                PlayerPostStage,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(player_camera)
                    .with_system(player_win_anim)
                    .with_system(player_win_sound)
                    .into(),
            );
    }
}

fn find_start_pos(start_q: Query<(&TilePos, &TileKind)>) -> Option<TilePos> {
    if start_q
        .iter()
        .filter(|(_, kind)| matches!(kind, TileKind::Start))
        .count()
        > 1
    {
        warn!("This level has more than one player start. Make sure, that your map file is correct.");
    }

    start_q
        .iter()
        .find(|(_, kind)| matches!(kind, TileKind::Start))
        .map(|(pos, _)| *pos)
}

pub fn spawn_player(
    mut commands: Commands,
    start_q: Query<(&TilePos, &TileKind)>,
    map_q: Query<(&Transform, &TilemapGridSize), With<MoveableTilemapTag>>,
    generated_assets: Res<GeneratedPlayerAssets>,
) {
    let (map_tf, map_grid) = match map_q.get_single() {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to query the level map: {}", e);
            return;
        }
    };

    let start_pos = match find_start_pos(start_q) {
        Some(x) => x,
        None => {
            error!("Start tile not found");
            return;
        }
    };

    let start_world_pos = tile_pos_to_world_pos(start_pos, map_tf, map_grid);

    commands
        .spawn((
            PlayerTag,
            Name::new("Player"),
            MoveableBundle::new(start_pos),
            MaterialMesh2dBundle {
                mesh: generated_assets.model.clone(),
                material: generated_assets.material.clone(),
                // TODO hardcoded player size
                // FIXME feels weird to double-set player's pos
                transform: Transform::from_translation(
                    start_world_pos.extend(moveable::MOVEABLE_Z_POS),
                )
                .with_scale(Vec3::new(16.0f32, 16.0f32, 16.0f32)),
                ..default()
            }
        ));
}
