use super::*;
use crate::moveable::*;
use crate::player::{PlayerTag, PlayerWinnerTag};
use anyhow::Context;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;

/// Switched the animation for tiles that have finished their
/// transition animation
pub fn tile_transition_anim_switch(
    mut graphics_q: Query<(&mut CPUAnimated, &GraphicsAnimating)>,
) {
    graphics_q.par_for_each_mut(10, |(mut animated, animating)| {
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
    graphics_map_q: Query<&TileStorage, With<GraphicsTilemapTag>>,
    logic_q: Query<(&LogicState, &TilePos), Changed<LogicState>>,
    mut graphics_q: Query<(Option<&mut CPUAnimated>, &GraphicsAnimating)>,
) {
    logic_q.for_each(|(state, tile_pos)| {
        // Redefine the function locally for easier usage
        let mut switch_animation = |entity| match graphics_q.get_mut(entity) {
            Ok((
                animated,
                animating,
            )) => switch_animation(
                commands.entity(entity),
                state,
                animating,
                animated,
            ),
            Err(e) => error!("Stale graphic tile for {tile_pos:?}: {e}"),
        };

        graphics_map_q.iter()
            .filter_map(|storage| storage.get(tile_pos))
            .for_each(&mut switch_animation)
    });
}

fn switch_animation(
    mut commands: EntityCommands,
    LogicState(logic_state): &LogicState,
    animating: &GraphicsAnimating,
    mut animated: Option<Mut<CPUAnimated>>,
) {
    // Animated that we will use (and later insert) if the entity doesn't have one already
    let mut new_animated = CPUAnimated::new(default(), false, false);
    let animation = if *logic_state {
        animating.on_transit.clone()
    } else {
        animating.off_transit.clone()
    };

    animated.as_deref_mut()
        .unwrap_or(&mut new_animated)
        .set_animation(animation, false, false);

    if animated.is_none() {
        commands.insert(new_animated);
    }
}

pub fn handle_button_triggers(
    mut tile_events: EventReader<TileEvent>,
    trigger_q: Query<(&ButtonCondition, &TilePos)>,
    logic_tilemap_q: Query<&TileStorage, With<LogicTilemapTag>>,
    mut logic_tile_q: Query<&mut LogicState>,
) {
    let logic_tilemap = match logic_tilemap_q.get_single() {
        Ok(x) => x,
        _ => return,
    };

    let mut apply_button_trigger = |pos, cond, button_id, entity| {
        match logic_tile_q.get_mut(entity) {
            Ok(mut tile_state) => apply_button_trigger(button_id, cond, &mut tile_state),
            Err(e) => error!("Stale logic tile for {pos:?}: {e}"),
        }
    };

    for ev in tile_events.iter() {
        let button_id = if let TileEvent::ButtonPressed { button_id } = ev {
            button_id
        } else {
            continue
        };

        trigger_q.iter()
            .filter_map(|(cond, pos)| logic_tilemap.get(pos).map(|entity| (pos, cond, entity)))
            .for_each(|(pos, cond, entity)| apply_button_trigger(pos, cond, *button_id, entity))
    }
}

fn apply_button_trigger(
    button_id: u8,
    cond: &ButtonCondition,
    tile_state: &mut LogicState,
) {
    if cond.is_active(button_id) && *tile_state != LogicState(true) {
        *tile_state = LogicState(true);
    }
}

/// Switches tile state, depending on which number the player has on their upper side.
/// This system makes sure to trigger `Changed` only when the state of a tile actually changes.
pub fn handle_player_side_triggers(
    player_q: Query<&Side, (With<PlayerTag>, Changed<Side>)>,
    trigger_q: Query<(&SideCondition, &TilePos)>,
    logic_tilemap_q: Query<&TileStorage, With<LogicTilemapTag>>,
    mut logic_tile_q: Query<&mut LogicState>,
) {
    let (player_side, logic_tilemap) = match (player_q.get_single(), logic_tilemap_q.get_single()) {
        (Ok(x), Ok(y)) => (x, y),
        _ => return,
    };

    // Redefine the function locally for easier usage
    let mut apply_side_trigger = |pos, cond, entity| {
        match logic_tile_q.get_mut(entity) {
            Ok(mut tile_state) => apply_side_trigger(player_side, cond, &mut tile_state),
            Err(e) => error!("Stale logic tile for {pos:?}: {e}"),
        }
    };

    trigger_q.iter()
        .filter_map(|(cond, pos)| logic_tilemap.get(pos).map(|entity| (pos, cond, entity)))
        .for_each(|(pos, cond, entity)| apply_side_trigger(pos, cond, entity))
}

fn apply_side_trigger(
    player_side: &Side,
    cond: &SideCondition,
    tile_state: &mut LogicState,
) {
    if let Side::Changing { from, to } = player_side {
        let next_state = cond.is_active(*to);

        // Do transition only when the state changes
        if cond.is_active(*from) != next_state {
            *tile_state = LogicState(next_state);
        }
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
        let (moveable, player_tag) = move_query.get_mut(interaction.moveable_id).context("Fetching moveable")?;
        let tile = tile_query.get_mut(interaction.tile_id).context("Fetching tile")?;

        handle_interaction(
            &mut commands,
            &mut tile_events,
            tile,
            moveable,
            player_tag.is_some(),
            interaction.moveable_id,
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
    is_moveable_player: bool,
    moveable_id: Entity,
) {
    use std::time::Duration;

    match &tile.kind {
        LogicKind::OnceButton(button_id) => if is_moveable_player && !tile.is_active() {
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
        LogicKind::Exit => if is_moveable_player && tile.is_active() {
            commands
                .entity(moveable_id)
                .remove::<MoveableBundle>()
                .insert(PlayerWinnerTag::new());
            tile_events.send(TileEvent::ExitReached);
        },
        _ => (),
    }
}