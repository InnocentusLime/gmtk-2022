use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::states::GameState;
use crate::player::{ PlayerMoved, PlayerMoveAcknowledged };

pub struct SpecialTilePlugin;

impl Plugin for SpecialTilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EndReached>()
            .add_event::<DangerReached>()
            .add_system(solid_tile_logic.run_in_state(GameState::InGame))
            .add_system(end_tile_logic.run_in_state(GameState::InGame));
    }
}

pub struct DangerReached;

pub struct EndReached;

#[derive(Component, Default)]
pub struct SolidTileTag;

#[derive(Component, Default)]
pub struct StartTileTag;

#[derive(Component, Default)]
pub struct AlwaysOnFrier;

#[derive(Component, Default)]
pub struct EndTileTag;

fn end_tile_logic(
    tiles: Query<&TilePos, With<EndTileTag>>,
    mut moves: EventReader<PlayerMoved>,
    mut level_end: EventWriter<EndReached>,
) {
    for ev in moves.iter() {
        if let Ok(_) = tiles.get(ev.cell) {
            level_end.send(EndReached);
        }
    }
}

fn solid_tile_logic(
    tiles: Query<&TilePos, With<SolidTileTag>>,
    mut moves: EventReader<PlayerMoved>,
    mut decisions: EventWriter<PlayerMoveAcknowledged>,
) {
    for ev in moves.iter() {
        if let Ok(_) = tiles.get(ev.cell) {
            decisions.send(PlayerMoveAcknowledged);
        }
    }
}

fn always_on_frier_logic(
    tiles: Query<&TilePos, With<AlwaysOnFrier>>,
    mut moves: EventReader<PlayerMoved>,
    mut decisions: EventWriter<DangerReached>,
) {
    for ev in moves.iter() {
        if let Ok(_) = tiles.get(ev.cell) {
            decisions.send(DangerReached);
        }
    }
}
