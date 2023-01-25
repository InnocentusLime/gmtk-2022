use bevy::prelude::*;
use cube_rot::MoveDirection;
use std::time::Duration;
use crate::GameplayCamera;
use crate::moveable::{ MoveableQuery, MoveableQueryItem };
use crate::tile::TileEvent;
use super::{ PlayerTag, PlayerWinnerTag, BasePlayerAssets };

pub fn player_win_sound(
    audio: Res<Audio>,
    assets: Res<BasePlayerAssets>,
    mut tile_events: EventReader<TileEvent>,
) {
    if tile_events.iter().filter_map(|x| match x {
            TileEvent::ExitReached => Some(()),
            _ => None,
        }).next().is_some()
    {
        audio.play(assets.complete_sound.clone());
    }
}

pub fn player_win_anim(
    time: Res<Time>,
    mut col_mats: ResMut<Assets<ColorMaterial>>,
    mut player_q: Query<(&mut Transform, &mut PlayerWinnerTag, &Handle<ColorMaterial>), With<PlayerTag>>,
) {
    player_q.for_each_mut(|(mut tf, mut win_tag, mat_handle)| {
        win_tag.timer.tick(time.delta());
        let t = win_tag.timer.percent_left();

        // TODO hardcoded player size
        tf.scale = Vec3::new(25.0f32, 25.0f32, 25.0f32) * t;
        col_mats.get_mut(mat_handle).unwrap().color = Color::Rgba {
            red: t,
            green: t,
            blue: t,
            alpha: 1.0,
        };
    });
}

pub fn player_camera(
    player_q: Query<&mut Transform, (With<PlayerTag>, Without<GameplayCamera>)>,
    mut gameplay_camera: Query<&mut Transform, With<GameplayCamera>>,
) {
    player_q.for_each(|player_tf| {
        gameplay_camera.single_mut().translation = player_tf.translation.truncate().extend(1000.0f32);
    })
}

#[derive(Default)]
pub struct InputQueue(Option<MoveDirection>);

fn check_input(key_input: &Res<Input<KeyCode>>) -> Option<MoveDirection> {
    if key_input.pressed(KeyCode::W) {
        return Some(MoveDirection::Up);
    }

    if key_input.pressed(KeyCode::A) {
        return Some(MoveDirection::Left);
    }

    if key_input.pressed(KeyCode::S) {
        return Some(MoveDirection::Down);
    }

    if key_input.pressed(KeyCode::D) {
        return Some(MoveDirection::Right);
    }

    None
}

fn player_flip(m: &mut MoveableQueryItem, dir: MoveDirection) -> bool {
    m.flip(dir, Duration::from_secs_f32(0.52f32))
}

/// The system for controlling the player. The system implements input
/// queueing.
///
/// # The queueing
/// The queueing works the following way:
///
/// 1. When the user presses a key when the cube is rolling -- the input
/// gets queued up.
/// 2. As the cube rolls, the user can change the queued up input by
/// pressing different keys.
/// 3. After the animation is over, the queued up input gets dequeued.
///
/// If the cube isn't rolling -- the input isn't queued up and gets applied
/// right away.
pub fn player_controls(
    mut queue: Local<InputQueue>,
    key_input: Res<Input<KeyCode>>,
    mut query: Query<MoveableQuery, With<PlayerTag>>,
) {
    /*
        Deque the input, but take it into account only if the player hasn't pressed
        any keys.
    */
    let queue_input = queue.0.take();
    let input = match check_input(&key_input).or(queue_input) {
        Some(x) => x,
        None => return,
    };

    let mut player = match query.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    /*
        If we fail to apply the input -- it gets queued up.

        Note, that it allows the input to come back into the queue.
    */
    if !player_flip(&mut player, input) {
        match player.movement_progress() {
            Some(x) if x >= 0.65f32 => {
                queue.0  = Some(input);
            },
            _ => (),
        }
    }
}
