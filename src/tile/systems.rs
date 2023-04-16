use super::*;
use crate::moveable::*;
use crate::player::{PlayerTag, PlayerWinnerTag};
use bevy::prelude::*;
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;

/// Switched the animation for tiles that have finished their
/// transition animation
pub fn tile_transition_anim_switch(
    mut graphics_q: Query<(&mut CPUAnimated, &GraphicsAnimating)>,
) {
    graphics_q.par_iter_mut()
    .for_each_mut(|(mut animated, animating)| {
        if !animated.is_done() {
            return;
        }

        let animation = if animated.animation() == &animating.on_transit {
            animating.on_anim.clone()
        } else {
            animating.off_anim.clone()
        };

        animated.set_animation(animation, false, true)
    })
}

/// Switches tile animation if its state changes.
pub fn tile_state_animation_switch(
    mut commands: Commands,
    mut tile_q: Query<(Entity, &LogicState, Option<&mut CPUAnimated>, &GraphicsAnimating), Changed<LogicState>>,
) {
    tile_q.for_each_mut(|(entity, state, mut animated, animating)| {
        let mut new_animated = CPUAnimated::new(default(), false, false);
        let animation = if state.0 {
            animating.on_transit.clone()
        } else {
            animating.off_transit.clone()
        };

        animated.as_deref_mut()
            .unwrap_or(&mut new_animated)
            .set_animation(animation, false, false);

        if animated.is_none() {
            commands.entity(entity).insert(new_animated);
        }
    });
}

pub fn handle_button_triggers(
    mut tile_events: EventReader<TileEvent>,
    mut tile_q: Query<(&ButtonCondition, &mut LogicState)>,
) {
    let button_events = tile_events.iter()
        .filter_map(|ev| match ev {
            TileEvent::ButtonPressed { button_id } => Some(*button_id),
            _ => None,
        });

    for button in button_events {
        tile_q.for_each_mut(|(cond, mut state, )| {
            if cond.is_active(button) && *state != LogicState(true) {
                *state = LogicState(true);
            }
        });
    }
}

/// Switches tile state, depending on which number the player has on their upper side.
/// This system makes sure to trigger `Changed` only when the state of a tile actually changes.
pub fn handle_player_side_triggers(
    player_q: Query<&Side, (With<PlayerTag>, Changed<Side>)>,
    mut tile_q: Query<(&SideCondition, &mut LogicState)>,
) {
    let (from, to) = match player_q.get_single() {
        Ok(&Side::Changing { from, to }) => (from, to),
        _ => return,
    };

    tile_q.for_each_mut(|(cond, mut state)| {
        let next_state = cond.is_active(to);

        // Do change only when the state changes
        if cond.is_active(from) != next_state {
            *state = LogicState(next_state);
        }
    });
}

pub fn special_tile_handler(
    mut interactions: EventReader<TileInteractionEvent>,
    mut tile_events: EventWriter<TileEvent>,
    mut tile_query: Query<LogicTileQuery>,
    mut move_query: Query<(MoveableQuery, Option<&PlayerTag>)>,
    mut commands: Commands,
) {
    for interaction in interactions.iter() {
        let (moveable, player_tag) = match move_query.get_mut(interaction.moveable_id) {
            Ok(x) => x,
            _ => continue,
        };
        let tile = match tile_query.get_mut(interaction.tile_id) {
            Ok(x) => x,
            _ => continue,
        };

        handle_interaction(
            &mut commands,
            &mut tile_events,
            tile,
            moveable,
            player_tag.is_some(),
            interaction.moveable_id,
        );
    }
}

fn handle_interaction(
    commands: &mut Commands,
    tile_events: &mut EventWriter<TileEvent>,
    mut tile: LogicTileQueryItem,
    mut moveable: MoveableQueryItem,
    with_player: bool,
    moveable_id: Entity,
) {
    use std::time::Duration;

    match &tile.kind {
        LogicKind::OnceButton(button_id) => if with_player && !tile.is_active() {
            *tile.state = LogicState(true);
            tile_events.send(TileEvent::ButtonPressed { button_id: *button_id });
        },
        LogicKind::Conveyor => if tile.is_active() {
            moveable.slide(tile.direction(), Duration::from_millis(500));
        },
        LogicKind::Frier => if tile.is_active() {
            commands.entity(moveable_id).despawn();
        },
        LogicKind::Spinner => if tile.is_active() {
            moveable.rotate(tile.is_clock_wise(), Duration::from_millis(500));
        },
        LogicKind::Exit => if with_player && tile.is_active() {
            commands
                .entity(moveable_id)
                .remove::<MoveableBundle>()
                .insert(PlayerWinnerTag::new());
            tile_events.send(TileEvent::ExitReached);
        },
        _ => (),
    }
}