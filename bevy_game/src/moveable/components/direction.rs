use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

#[derive(Debug, Clone, Copy, Default, Inspectable)]
#[repr(u8)]
pub enum MoveDirection {
    #[default]
    Up = 0,
    Left = 1,
    Down = 2,
    Right = 3,
}

impl MoveDirection {
    pub fn to_offset(self) -> (i32, i32) {
        match self {
            Self::Up => (0, 1),
            Self::Left => (-1, 0),
            Self::Down => (0, -1),
            Self::Right => (1, 0),
        }
    }

    pub fn apply_on_pos(self, (x, y): (u32, u32)) -> Option<(u32, u32)> {
        let (dx, dy) = self.to_offset();
        let (x, y) = (x as i32 + dx, y as i32 + dy);

        if x < 0 || y < 0 {
            None
        } else {
            Some((x as u32, y as u32))
        }
    }

    pub fn to_quat(self, t: f32) -> Quat {
        match self {
            Self::Left => Quat::from_rotation_y(-t * std::f32::consts::FRAC_PI_2),
            Self::Right => Quat::from_rotation_y(t * std::f32::consts::FRAC_PI_2),
            Self::Up => Quat::from_rotation_x(-t * std::f32::consts::FRAC_PI_2),
            Self::Down => Quat::from_rotation_x(t * std::f32::consts::FRAC_PI_2),
        }
    }

    /// Flip direction along x axis
    pub fn flip_x(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            _ => self,
        }
    }

    /// Flip direction along y axis
    pub fn flip_y(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            _ => self,
        }
    }

    /// Flip direction anti-diagonally
    pub fn flip_d(self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Left => Self::Down,
            Self::Down => Self::Left,
            Self::Right => Self::Up,
        }
    }

    /// Apply the flipping flags
    pub fn apply_flipping_flags(mut self, flip_x: bool, flip_y: bool, flip_d: bool) -> Self {
        if flip_d { self = self.flip_d() }
        if flip_y { self = self.flip_y() }
        if flip_x { self = self.flip_x() }

        self
    }
}
