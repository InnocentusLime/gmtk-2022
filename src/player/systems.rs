use bevy::prelude::*;
use cube_rot::MoveDirection;
use std::time::Duration;
use crate::GameplayCamera;
use crate::moveable::{ MoveableQuery, MoveableQueryItem };
use super::{ PlayerTag, PlayerWinnerTag, PlayerEscapedEvent, BasePlayerAssets };

pub fn player_win_sound(
    audio: Res<Audio>,
    assets: Res<BasePlayerAssets>,
    mut events: EventReader<PlayerEscapedEvent>,
) {
    for _ in events.iter() {
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
        gameplay_camera.single_mut().translation = player_tf.translation.truncate().extend(50.0f32);
    })
}

#[derive(Default)]
pub struct InputQueue(Option<MoveDirection>);

pub fn player_controls(
    mut queue: Local<InputQueue>,
    key_input: Res<Input<KeyCode>>,
    mut query: Query<MoveableQuery, With<PlayerTag>>,
) {
    use crate::moveable::MoveDirection::*;

    let player_flip = |mut m: MoveableQueryItem, dir| m.flip(dir, Duration::from_secs_f32(0.52f32));
   
    // TODO pretify?
    let mut movement = None;
    if key_input.pressed(KeyCode::W) { movement = movement.or(Some(Up)); } 
    if key_input.pressed(KeyCode::A) { movement = movement.or(Some(Left)); }
    if key_input.pressed(KeyCode::S) { movement = movement.or(Some(Down)); }
    if key_input.pressed(KeyCode::D) { movement = movement.or(Some(Right)); }

    match movement {
        Some(dir) => query.for_each_mut(|m| { 
            // Ignore inputs, that happened too early
            match m.movement_progress() {
                Some(x) if x <= 0.1f32 => return, 
                _ => (),
            }

            if !player_flip(m, dir) {
                queue.0 = movement;
            } 
        }),
        None => if let Some(dir) = queue.0 {
            query.for_each_mut(|m| {
                if player_flip(m, dir) {
                    queue.0 = None;
                }
            });
        },
    }
}
