use bevy::prelude::*;
use bevy::ecs::query::{ WorldQuery, Fetch };
use iyes_loopless::prelude::*;

use super::cpu_tile_animation::CPUAnimated;

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

#[derive(Clone, Copy)]
pub enum ActivatableAnimating {
    // Loops the `on_anim`, without looping the
    // `off_anim`
    //
    // anim_type = switchable_machine
    Switch {
        on_anim: CPUAnimated,
        off_anim: CPUAnimated,
    },
    // Loops the anim when the tile is on and
    // stops it when off
    //
    // anim_type = stoppable_machine
    Stop {
        on_speed: f32
    },
}

impl ActivatableAnimating {
    fn update_cpu_anim(&self, anim: &mut CPUAnimated, active: bool) {
        use ActivatableAnimating::*;
        match self {
            Switch { on_anim, off_anim } => if active {
                *anim = *on_anim;
            } else {
                *anim = *off_anim;
            },
            Stop { on_speed } => if active {
                anim.set_speed(*on_speed);
            } else {
                anim.set_speed(0.0f32);
            },
        }
    }
}

#[derive(Component)]
pub struct ActivatableTileTag {
    state: bool,
    condition: ActivationCondition,
    anim_info: ActivatableAnimating,
}

impl ActivatableTileTag {
    pub fn new(
        condition: ActivationCondition,
        anim_info: ActivatableAnimating,
    ) -> Self {
        ActivatableTileTag {
            state: condition.active_on_start(),
            condition, anim_info,
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

fn toggle_activatable_tiles<F>(
    mut filter: F,
    mut query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>, 
) 
where
    F: FnMut(&mut ActivatableTileTag) -> bool
{
    for (mut cpu_anim, mut active_tag) in query.iter_mut() {
        if filter(&mut *active_tag) {
            active_tag.anim_info.update_cpu_anim(&mut *cpu_anim, active_tag.is_active());
        }
    }
}

// TODO rename to setup
pub fn activeatable_tile_setup_system(query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>) {
    toggle_activatable_tiles(|_| true, query);
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

pub fn tile_switch_system(
    mut moves: EventReader<PlayerMoved>,
    query: Query<(&mut CPUAnimated, &mut ActivatableTileTag)>, 
) {
    if let Some(e) = moves.iter().next() {
        toggle_activatable_tiles(|tag| tag.update(&e.dice_state), query);
    }

    // Drop all other events
    moves.iter().for_each(|_| ());
}

pub trait AddTile {
    fn add_tile<T: TileState>(&mut self) -> &mut Self;
}

impl AddTile for App {
    fn add_tile<T: TileState>(&mut self) -> &mut Self {
        self.add_system(tile_reaction_system::<T>.run_in_state(GameState::InGame))
    }
}
