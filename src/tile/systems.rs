use super::*;
use crate::moveable::*;
use crate::player::{PlayerTag, PlayerWinnerTag};
use anyhow::Context;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;

/// Switches tile animation if its state changes. See [ActivatableAnimating] for more info.
pub fn tile_animation_switch(
    mut commands: Commands,
    graphics_map_q: Query<&TileStorage, With<GraphicsTilemapTag>>,
    logic_q: Query<(&TileState, &TilePos), Changed<TileState>>,
    mut graphics_q: Query<(Option<&mut CPUAnimated>, &ActivatableAnimating)>,
) {
    logic_q.for_each(|(state, tile_pos)| {
        let mut try_switch_animation = |entity| match graphics_q.get_mut(entity) {
            Ok((
                animated,
                animating,
            )) => switch_animation_opt(
                commands.entity(entity),
                state,
                animating,
                animated,
            ),
            Err(e) => error!("Stale graphic tile for {tile_pos:?}: {e}"),
        };

        graphics_map_q.iter()
            .filter_map(|storage| storage.get(tile_pos))
            .for_each(&mut try_switch_animation)
    });
}

fn switch_animation_opt(
    mut commands: EntityCommands,
    state: &TileState,
    animating: &ActivatableAnimating,
    mut animated: Option<Mut<CPUAnimated>>,
) {
    // Animated that we will use (and later insert) if the entity doesn't have one already
    let mut new_animated = CPUAnimated::new(default(), false, false);

    switch_animation(
        state,
        animating,
        animated.as_deref_mut().unwrap_or(&mut new_animated)
    );

    if animated.is_none() {
        commands.insert(new_animated);
    }
}

fn switch_animation(
    state: &TileState,
    animating: &ActivatableAnimating,
    animated: &mut CPUAnimated,
) {
    match (state, animating) {
        (
            TileState::Ready(x),
            ActivatableAnimating::Switch { on_anim, off_anim, .. }
        ) => animated.set_animation(
            if *x { on_anim } else { off_anim }.clone(),
            false,
            true,
        ),
        (
            TileState::Ready(x),
            ActivatableAnimating::Pause { on_anim },
        ) => animated.set_animation(
            on_anim.clone(),
            !*x,
            true,
        ),
        (
            TileState::Changing { to },
            ActivatableAnimating::Switch { on_transit, off_transit, .. },
        ) => animated.set_animation(
            if *to { on_transit } else { off_transit }.clone(),
            false,
            false
        ),
        _ => (),
    }
}

/// Switches tile state, depending on which number the player has on their upper side.
/// This system makes sure to trigger `Changed` only when the state of a tile actually changes.
pub fn tile_state_switching(
    player_q: Query<&Side, (With<PlayerTag>, Changed<Side>)>,
    trigger_q: Query<(&ActivationCondition, &TilePos)>,
    logic_tilemap_q: Query<&TileStorage, With<LogicTilemapTag>>,
    mut logic_tile_q: Query<&mut TileState>,
) {
    let (player_side, logic_tilemap) = match (player_q.get_single(), logic_tilemap_q.get_single()) {
        (Ok(x), Ok(y)) => (x, y),
        _ => return,
    };
    let mut try_update_tile_state = |pos, cond, entity| {
        match logic_tile_q.get_mut(entity) {
            Ok(mut tile_state) => update_tile_state(player_side, cond, &mut tile_state),
            Err(e) => error!("Stale logic tile for {pos:?}: {e}"),
        }
    };

    trigger_q.iter()
        .filter_map(|(cond, pos)|
            logic_tilemap.get(pos).map(|entity| (pos, cond, entity))
        )
        .for_each(|(pos, cond, entity)| try_update_tile_state(pos, cond, entity))
}

fn update_tile_state(
    player_side: &Side,
    cond: &ActivationCondition,
    tile_state: &mut TileState,
) {
    match player_side {
        Side::Ready(upper_side) => {
            let new_state = TileState::Ready(cond.is_active(*upper_side));

            // Don't trip change detection when we aren't actually changing the state
            if new_state != *tile_state {
                *tile_state = new_state;
            }
        },
        Side::Changing { from, to } => {
            let next_state = cond.is_active(*to);

            // Do transition only when the state changes
            if cond.is_active(*from) != next_state {
                *tile_state = TileState::Changing { to: next_state };
            }
        },
    }
}

pub fn special_tile_handler(
    mut interactions: EventReader<TileInteractionEvent>,
    mut tile_events: EventWriter<TileEvent>,
    mut tile_query: Query<LogicTileQuery>,
    mut move_query: Query<(MoveableQuery, Option<&PlayerTag>)>,
    mut commands: Commands,
) {
    let mut try_handle_interaction = |interaction: &TileInteractionEvent| {
        let (moveable, player_tag) = move_query.get_mut(
            interaction.moveable_id
        ).context("Fetching moveable")?;
        let tile = tile_query.get_mut(
            interaction.tile_id
        ).context("Fetching tile")?;

        handle_interaction(
            &mut commands,
            &mut tile_events,
            tile,
            moveable,
            player_tag.is_some(),
            interaction.moveable_id,
            interaction.tile_id,
        );

        anyhow::Ok(())
    };

    for interaction in interactions.iter() {
        if let Err(e) = try_handle_interaction(interaction) {
            error!("Error while handling interaction: {e:?}");
        }
    }
}

fn handle_interaction(
    commands: &mut Commands,
    tile_events: &mut EventWriter<TileEvent>,
    mut tile: LogicTileQueryItem,
    mut moveable: MoveableQueryItem,
    moveable_is_player: bool,
    moveable_id: Entity,
    tile_id: Entity,
) {
    use std::time::Duration;

    match &tile.kind {
        TileKind::OnceButton => if moveable_is_player && !tile.is_active() {
            *tile.state = TileState::Ready(true);
            tile_events.send(TileEvent::ButtonPressed { tile_id });
        },
        TileKind::Conveyor => if tile.is_active() {
            moveable.slide(tile.direction(), Duration::from_millis(500));
        },
        TileKind::Frier => if tile.is_active() {
            commands.entity(moveable_id).despawn();
        },
        TileKind::Spinner => if tile.is_active() {
            moveable.rotate(tile.is_clock_wise(), Duration::from_millis(500));
        },
        TileKind::Exit => if moveable_is_player && tile.is_active() {
            commands
                .entity(moveable_id)
                .remove::<MoveableBundle>()
                .insert(PlayerWinnerTag::new());
            tile_events.send(TileEvent::ExitReached);
        },
        _ => (),
    }
}