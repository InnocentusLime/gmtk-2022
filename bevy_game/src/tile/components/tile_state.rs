use bevy::prelude::*;
use bevy::ecs::query::{ WorldQuery, Fetch };

use crate::player::PlayerModification;

pub type QueryMutElem<'w, 's, Q> = <<Q as WorldQuery>::Fetch as Fetch<'w, 's>>::Item;

pub trait TileState: Sized + Component {
    type UpdateData: WorldQuery;

    fn react<'w, 's>(&mut self, extra: QueryMutElem<'w, 's, Self::UpdateData>) -> PlayerModification;
}
