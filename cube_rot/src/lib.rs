//use std::num::NonZeroU8;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
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

    pub fn apply_on_pos(self, TilePos { x, y }: TilePos) -> Option<TilePos> {
        let (dx, dy) = self.to_offset();
        let (x, y) = (x as i32 + dx, y as i32 + dy);

        if x < 0 || y < 0 {
            None
        } else {
            Some(TilePos { x: x as u32, y: y as u32 })
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


// Reimplementation draft
/*
#[derive(Clone, Copy, Debug)]
enum FlatRotation {
    // No rotation
    None,
    // Rotate to the right
    Horizontal(NonZeroU8),
    // Rotate down
    Vertical(NonZeroU8),
}

impl FlatRotation {
    fn compose_horiz(hor: NonZeroU8, other: FlatRotation) -> (u8, FlatRotation) {
        match other {
            Self::None => (0, Self::Horizontal(hor)),
            Self::Horizontal(d) => (0, NonZeroU8::new((d.get() + hor) % 4)
                .map(Self::Horizontal)
                .unwrap_or(Self::None)),
            Self::Vertical(d) => (
                (hor.get() * (d.get() + 6)) % 4,
                NonZeroU8::new((3 * hor.get() * (1 - d.get() % 2)) % 4)
                    .map(Self::Vertical)
                    .unwrap_or(Self::None)
            ),
        }
    }

    fn compose_vert(vert: NonZeroU8, other: FlatRotation) -> (u8, FlatRotation) {
        todo!()
    }

    /// Composes two flat rotations, returning the result
    /// and the additional orthogonal rotation. The orthogonal
    /// rotation is considered clockwise.
    fn compose(self, other: FlatRotation) -> (u8, FlatRotation) {
        match self {
            Self::None => (0, other),
            Self::Horizontal(x) => Self::compose_horiz(x, other),
            Self::Vertical(x) => Self::compose_vert(x, other),
        }
    }
}
*/

#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
enum BoxSide {
    Up = 0,
    Left = 1,
    Down = 2,
    Right = 3,
    Bottom = 4,
    #[default]
    Top = 5,
}

impl BoxSide {
    fn to_quat(self) -> Quat {
        match self {
            Self::Left => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            Self::Right => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Self::Up => Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            Self::Down => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            Self::Top => Quat::IDENTITY,
            Self::Bottom => Quat::from_rotation_x(std::f32::consts::PI),
        }
    }
}

/// Represents object rotation using integers, assuming it's always going
/// to be rotated by 90 degreees.
#[derive(Debug, Clone, Copy, Default)]
pub struct DecomposedRotation {
    ortho_rot: u8, // Orthogonal rotation clock-wise. Gets applied first
    flat_rot: BoxSide, // Flat rotation. Gets applied second
}

impl DecomposedRotation {
    fn map_side(side: u8) -> u8 {
        static INITIAL_DICE_TABLE: [u8; 6] = [5, 6, 2, 1, 3, 4];
        INITIAL_DICE_TABLE[side as usize]
    }

    fn rot_comp(x: BoxSide, y: BoxSide) -> (u8, BoxSide) {
        use BoxSide::*;
        static MUL_TABLE: [[(u8, BoxSide); 6]; 6] = {
            let (u, l, d, r, b, e) = (Up, Left, Down, Right, Bottom, Top);
            [
                [(0, b), (3, l), (0, e), (1, r), (0, d), (0, u)],
                [(1, u), (2, b), (3, d), (0, e), (2, r), (0, l)],
                [(0, e), (1, l), (0, b), (3, r), (0, u), (0, d)],
                [(3, u), (0, e), (1, d), (2, b), (2, l), (0, r)],
                [(0, d), (2, l), (0, u), (2, r), (0, e), (0, b)],
                [(0, u), (0, l), (0, d), (0, r), (0, b), (0, e)],
            ]
        };

        MUL_TABLE[y as usize][x as usize]
    }

    fn dir_to_rot(x: MoveDirection) -> BoxSide {
        static MAP_TABLE: [BoxSide; 4] = [
            BoxSide::Up,
            BoxSide::Left,
            BoxSide::Down,
            BoxSide::Right,
        ];

        MAP_TABLE[x as usize]
    }

    pub fn new() -> Self {
        Self {
            ortho_rot: 0,
            flat_rot: BoxSide::Top,
        }
    }

    pub fn upper_side(&self) -> u8 {
        let rot_id = self.flat_rot as u8;

        if rot_id < 4 {
            Self::map_side((rot_id + self.ortho_rot) % 4)
        } else {
            Self::map_side(rot_id)
        }
    }

    #[must_use]
    pub fn rotate_in_dir(&self, d: MoveDirection) -> Self {
        let (delta_ortho_rot, flat_rot) = Self::rot_comp(self.flat_rot, Self::dir_to_rot(d));

        Self {
            ortho_rot: (self.ortho_rot + delta_ortho_rot) % 4,
            flat_rot,
        }
    }

    #[must_use]
    pub fn rotate_ortho(&self, clock_wise: bool) -> Self {
        if clock_wise {
            self.rotate_in_dir(MoveDirection::Up)
                .rotate_in_dir(MoveDirection::Left)
                .rotate_in_dir(MoveDirection::Down)
        } else {
            self.rotate_in_dir(MoveDirection::Up)
                .rotate_in_dir(MoveDirection::Right)
                .rotate_in_dir(MoveDirection::Down)
        }
    }

    pub fn rot_quat(&self) -> Quat {
        let flat_rot_quat = self.flat_rot.to_quat();
        let ortho_rot_quat = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2 * self.ortho_rot as f32);

        flat_rot_quat * ortho_rot_quat
    }
}
