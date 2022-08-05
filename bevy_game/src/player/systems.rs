use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::level::tile_pos_to_world_pos;
use bevy::input::keyboard::KeyboardInput;

use std::time::Duration;
use crate::app::GameplayCamera;
use crate::moveable::Moveable;
use super::{ PlayerTag };

/*
 
            let t = timer.percent_left();
            // TODO hardcoded player size
            player_tf.scale = Vec3::new(25.0f32, 25.0f32, 25.0f32) * t;
            col_mats.get_mut(&*mat_handle).unwrap().color = Color::Rgba {
                red: t,
                green: t,
                blue: t,
                alpha: 1.0,
            };
*/

/*
pub fn player_sound(
    audio: Res<Audio>,
    assets: Res<BasePlayerAssets>,
    mut events: EventReader<PlayerModification>,
) {
    for e in events.iter() {
        match e {
            PlayerModification::Escape => {
                audio.play(assets.complete_sound.clone());
            },
            _ => (),
        }
    }
}
*/

pub fn player_camera(
    mut player_q: Query<&mut Transform, (With<PlayerTag>, Without<GameplayCamera>)>,
    mut gameplay_camera: Query<&mut Transform, With<GameplayCamera>>,
) {
    for player_tf in player_q.iter_mut() {
        gameplay_camera.single_mut().translation = player_tf.translation.truncate().extend(50.0f32);
    }
}

pub fn player_controls(
    mut key_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Moveable, With<PlayerTag>>,
) {
    use crate::moveable::MoveDirection::*;
   
    // TODO pretify?
    let mut movement = None;
    if key_input.pressed(KeyCode::W) { movement = Some(Up); } 
    if key_input.pressed(KeyCode::A) { movement = Some(Left); }
    if key_input.pressed(KeyCode::S) { movement = Some(Down); }
    if key_input.pressed(KeyCode::D) { movement = Some(Right); }

    if let Some(dir) = movement {
        query.for_each_mut(|mut m| { m.flip(dir, Duration::from_secs_f32(0.52f32)); });
    }
}
