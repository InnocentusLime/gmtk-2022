use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::states::GameState;
use crate::player::{ MovePlayer, PlayerMoved, PlayerMoveAcknowledged };

pub struct SpecialTilePlugin;

impl Plugin for SpecialTilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<EndReached>()
            .add_event::<DangerReached>()
            .add_system(solid_tile_logic.run_in_state(GameState::InGame))
            .add_system(end_tile_logic.run_in_state(GameState::InGame))
            .add_system_set(
                SystemSet::new()
                    .with_system(conveyor_logic.run_in_state(GameState::InGame))
                    .with_system(conveyor_condition_logic.run_in_state(GameState::InGame).before(conveyor_logic))
            )
            .add_system_set(
                SystemSet::new()
                    .with_system(fry_logic.run_in_state(GameState::InGame))
                    .with_system(fry_condition_logic.run_in_state(GameState::InGame).before(fry_logic))
            );
    }
}

#[derive(Clone, Copy)]
pub enum ActivationCondition {
    Odd,
    Even,
}

impl ActivationCondition {
    pub fn active_on_start(self) -> bool {
        match self {
            ActivationCondition::Odd => false,
            ActivationCondition::Even => true,
        }
    }

    fn is_active(self, state: &crate::player::dice::DiceEncoding) -> bool {
        let side = state.upper_side();
        match self {
            ActivationCondition::Odd => side % 2 == 1,
            ActivationCondition::Even => side % 2 == 0,
        }
    }
}

pub struct DangerReached;

pub struct EndReached;

#[derive(Component, Clone, Copy)]
pub struct ConveyorTag {
    pub active: bool,
    pub cond: ActivationCondition,
}

#[derive(Component, Default)]
pub struct SolidTileTag;

#[derive(Component, Default)]
pub struct StartTileTag;

#[derive(Component)]
pub struct FrierTag {
    pub active: bool,
    pub cond: ActivationCondition,
}

#[derive(Component, Default)]
pub struct EndTileTag;

fn fry_condition_logic(
    mut commands: Commands,
    mut tiles: Query<(Entity, &mut FrierTag, &TilePos, &mut Tile)>,
    mut moves: EventReader<PlayerMoved>,
    mut map: MapQuery,
) {
    for ev in moves.iter() {
        for (e, mut tag, pos, mut tile_dat) in tiles.iter_mut() {
            let active = tag.cond.is_active(&ev.dice_state);
            if !active && tag.active {
                tag.active = false;
                tile_dat.texture_index = 4;
                map.notify_chunk_for_tile(*pos, 0u16, 0u16);
            } else if active && !tag.active {
                tag.active = true;
                tile_dat.texture_index = 8;
                map.notify_chunk_for_tile(*pos, 0u16, 0u16);
            }
        }
    }
}

fn conveyor_condition_logic(
    mut commands: Commands,
    mut tiles: Query<(Entity, &mut ConveyorTag, &TilePos, &Tile)>,
    mut moves: EventReader<PlayerMoved>,
    mut map: MapQuery,
) {
    for ev in moves.iter() {
        for (e, mut tag, pos, tile_dat) in tiles.iter_mut() {
            let active = tag.cond.is_active(&ev.dice_state);
            if !active && tag.active {
                tag.active = false;
                commands.entity(e).remove::<GPUAnimated>();
                map.notify_chunk_for_tile(*pos, 0u16, 0u16);
            } else if active && !tag.active {
                tag.active = true;
                commands.entity(e).insert(GPUAnimated::new(0, 4, 8.0f32));
                map.notify_chunk_for_tile(*pos, 0u16, 0u16);
            }
        }
    }
}

fn conveyor_logic(
    tiles: Query<(&TilePos, &ConveyorTag)>,
    mut moves: EventReader<PlayerMoved>,
    mut decisions: EventWriter<MovePlayer>,
    mut decisions2: EventWriter<PlayerMoveAcknowledged>,
) {
    for ev in moves.iter() {
        if let Ok((_, tag)) = tiles.get(ev.cell) {
            if tag.active {
                decisions.send(MovePlayer {
                    spin: false,
                    dir: crate::player::dice::Direction::Up,
                });
            } else {
                decisions2.send(PlayerMoveAcknowledged);
            }
        }
    }
}

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

fn fry_logic(
    tiles: Query<(&TilePos, &FrierTag)>,
    mut moves: EventReader<PlayerMoved>,
    mut decisions: EventWriter<DangerReached>,
    mut decisions2: EventWriter<PlayerMoveAcknowledged>,
) {
    for ev in moves.iter() {
        if let Ok((_, tag)) = tiles.get(ev.cell) {
            if tag.active {
                decisions.send(DangerReached);
            } else {
                decisions2.send(PlayerMoveAcknowledged);
            }
        }
    }
}
