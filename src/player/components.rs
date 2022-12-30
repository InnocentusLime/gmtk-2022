use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Component)]
pub struct PlayerTag;

#[derive(Clone, Debug, Component)]
pub struct PlayerWinnerTag {
    pub(super) timer: Timer,
}

impl PlayerWinnerTag {
    pub fn new() -> Self {
        PlayerWinnerTag {
            timer: Timer::from_seconds(0.31f32, TimerMode::Once),
        }
    }
}
