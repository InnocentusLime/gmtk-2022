use bevy::prelude::*;
use bevy::ecs::query::{ WorldQuery, Fetch };
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::states::GameState;
use crate::player::dice::DiceEncoding;
use crate::player::{ PlayerModification, PlayerMoved };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StageLabel)]
pub struct ActiveTileUpdateStage;

pub struct MapReady;

#[derive(Clone, Copy)]
pub enum ActivationCondition {
    Odd,
    Even,
}

impl ActivationCondition {
    // TODO don't hardcode the starting side
    pub fn active_on_start(self) -> bool {
        match self {
            ActivationCondition::Odd => false,
            ActivationCondition::Even => true,
        }
    }

    fn is_active(self, player_state: &DiceEncoding) -> bool {
        let side = player_state.upper_side();
        match self {
            ActivationCondition::Odd => side % 2 == 1,
            ActivationCondition::Even => side % 2 == 0,
        }
    }
}

#[derive(Component)]
pub struct ActivatableTileTag {
    state: bool,
    condition: ActivationCondition,
}

impl ActivatableTileTag {
    pub fn new(condition: ActivationCondition) -> Self {
        ActivatableTileTag {
            state: condition.active_on_start(),
            condition,
        }
    }

    pub fn is_active(&self) -> bool { self.state }

    /// Updated the state of the tile, returning `true` if the internal
    /// logic needs to be updated.
    fn update(&mut self, player_state: &DiceEncoding) -> bool {
        let new_state = self.condition.is_active(player_state);
        let result = self.state != new_state;
        self.state = new_state;
        result
    }
}

pub type QueryMutElem<'w, 's, Q> = <<Q as WorldQuery>::Fetch as Fetch<'w, 's>>::Item;

pub trait TileState: Sized + Component {
    type UpdateData: WorldQuery;

    fn react<'w, 's>(&mut self, extra: QueryMutElem<'w, 's, Self::UpdateData>) -> PlayerModification;
}

/// Extra state data for the activatable tiles.
pub trait ActivatableTileState: TileState {
    type SwitchData: bevy::ecs::query::WorldQuery;

    /// Method to setup the internal data of the tile to be enabled. It is
    /// guaranteed, that it will be called only when the tile **hasn't been** enabled yet.
    fn enable<'w, 's>(&mut self, commands: &mut Commands, extra: QueryMutElem<'w, 's, Self::SwitchData>);
    
    /// Method to setup the internal data of the tile to be disabled. It is
    /// guaranteed, that it will be called only when the tile **hasn't been** disabled yet.
    fn disable<'w, 's>(&mut self, commands: &mut Commands, extra: QueryMutElem<'w, 's, Self::SwitchData>);
}

pub fn activeatable_tile_setup_system<T: ActivatableTileState>(
    mut query: Query<(&mut ActivatableTileTag, &TilePos, &mut T, T::SwitchData)>, 
    mut commands: Commands,
    mut map: MapQuery,
) {
    for (active_tag, pos, mut state, extra) in query.iter_mut() {
        if active_tag.is_active() {
            state.enable(&mut commands, extra);
        } else {
            state.disable(&mut commands, extra);
        }
        map.notify_chunk_for_tile(*pos, 0u16, 0u16);
    }
}

pub fn tile_reaction_system<T: TileState>(
    mut moves: EventReader<PlayerMoved>,
    mut query: Query<(&mut T, T::UpdateData)>,
    mut response: EventWriter<PlayerModification>,
) {
    use std::any::type_name;
    for e in moves.iter() {
        if let Ok((mut state, extra)) = query.get_mut(e.cell) {
            trace!("{} reacted to player", type_name::<T>());
            response.send(state.react(extra));
        }
    }
}

pub fn tile_switch_system<T: ActivatableTileState>(
    mut moves: EventReader<PlayerMoved>,
    mut query: Query<(&mut ActivatableTileTag, &TilePos, &mut T, T::SwitchData)>, 
    mut commands: Commands,
    mut map: MapQuery,
) {
    for e in moves.iter() {
        for (mut active_tag, pos, mut state, extra) in query.iter_mut() {
            if active_tag.update(&e.dice_state) {
                if active_tag.is_active() {
                    state.enable(&mut commands, extra);
                } else {
                    state.disable(&mut commands, extra);
                }
                map.notify_chunk_for_tile(*pos, 0u16, 0u16);
            }
        }
    }
}

pub trait AddTile {
    fn add_tile<T: TileState>(&mut self) -> &mut Self;
    fn add_activatable_tile<T: ActivatableTileState>(&mut self) -> &mut Self;
}

impl AddTile for App {
    fn add_tile<T: TileState>(&mut self) -> &mut Self {
        self.add_system(tile_reaction_system::<T>.run_in_state(GameState::InGame))
    }
    
    fn add_activatable_tile<T: ActivatableTileState>(&mut self) -> &mut Self {
        self
            .add_tile::<T>()
            .add_system_to_stage(ActiveTileUpdateStage, tile_switch_system::<T>.run_in_state(GameState::InGame))
            .add_system(
                activeatable_tile_setup_system::<T>.run_in_state(GameState::Spawning)
                .run_on_event::<MapReady>()
            )
    }
}
