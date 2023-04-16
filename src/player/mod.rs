mod components;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_ecs_tilemap::prelude::*;

use crate::level::tile_pos_to_world_pos;
use crate::moveable::{MoveableBundle, MoveableTilemapTag, self};
use crate::states::GameState;
use crate::tile::{LogicKind, TileSystems};

pub use components::*;
pub use resources::*;
pub use systems::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum PlayerSystems {
    Controls,
    PostUpdate,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<PlayerTag>()
            .register_type::<PlayerWinnerTag>()
            .add_system(
                player_controls
                .in_set(PlayerSystems::Controls)
                .after(TileSystems::InteractionHandler)
                .run_if(in_state(GameState::InGame))
            )
            .add_systems(
                (player_camera,)
                .in_base_set(CoreSet::PostUpdate)
                .in_set(PlayerSystems::PostUpdate)
                .distributive_run_if(in_state(GameState::InGame))
            );
    }
}

fn find_start_pos(start_q: Query<(&TilePos, &LogicKind)>) -> Option<TilePos> {
    if start_q
        .iter()
        .filter(|(_, kind)| matches!(kind, LogicKind::Start))
        .count()
        > 1
    {
        warn!("This level has more than one player start. Make sure, that your map file is correct.");
    }

    start_q
        .iter()
        .find(|(_, kind)| matches!(kind, LogicKind::Start))
        .map(|(pos, _)| *pos)
}

pub fn spawn_player(
    mut commands: Commands,
    start_q: Query<(&TilePos, &LogicKind)>,
    map_q: Query<(&Transform, &TilemapGridSize), With<MoveableTilemapTag>>,
    //generated_assets: Res<GeneratedPlayerAssets>,
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

    info!("Spawning player at {start_pos:?}");
    commands
        .spawn((
            PlayerTag,
            Name::new("Player"),
            MoveableBundle::new(start_pos),
            // MaterialMesh2dBundle {
            //     mesh: generated_assets.model.clone(),
            //     material: generated_assets.material.clone(),
            //     // TODO hardcoded player size
            //     // FIXME feels weird to double-set player's pos
            //     transform: Transform::from_translation(
            //         start_world_pos.extend(moveable::MOVEABLE_Z_POS),
            //     )
            //     .with_scale(Vec3::new(16.0f32, 16.0f32, 16.0f32)),
            //     ..default()
            // }
        ));
}
