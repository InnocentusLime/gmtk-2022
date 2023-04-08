use bevy::prelude::*;

#[derive(Clone, Copy, Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerTag;

#[derive(Clone, Debug, Component, Reflect)]
pub struct PlayerWinnerTag {
    pub(super) timer: Timer,
}

impl Default for PlayerWinnerTag {
    fn default() -> Self { Self::new() }
}

impl PlayerWinnerTag {
    pub fn new() -> Self {
        PlayerWinnerTag {
            timer: Timer::from_seconds(0.31f32, TimerMode::Once),
        }
    }
}
