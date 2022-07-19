use bevy::prelude::Quat;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Direction {
    Up = 0,
    Left = 1,
    Down = 2,
    Right = 3,
}

impl Direction {
    pub fn to_offset(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Down => (0, -1),
            Direction::Right => (1, 0),
        }
    }

    pub fn to_quat(self, t: f32) -> Quat {
        match self {
            Direction::Left => Quat::from_rotation_y(-t * std::f32::consts::FRAC_PI_2),
            Direction::Right => Quat::from_rotation_y(t * std::f32::consts::FRAC_PI_2),
            Direction::Up => Quat::from_rotation_x(-t * std::f32::consts::FRAC_PI_2),
            Direction::Down => Quat::from_rotation_x(t * std::f32::consts::FRAC_PI_2),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum FlatDiceRot {
    ToUp = 0,
    ToLeft = 1,
    ToDown = 2,
    ToRight = 3,
    ToBottom = 4,
    ToTop = 5,
}

/// Represenation of dice rotation.
#[derive(Clone, Copy)]
pub struct DiceEncoding {
    ortho_rot: u8, // Orthogonal rotation clock-wise. Gets applied first
    flat_rot: FlatDiceRot, // Flat rotation. Gets applied second
}

impl DiceEncoding {
    fn map_side(side: u8) -> u8 {
        static INITIAL_DICE_TABLE: [u8; 6] = [5, 6, 2, 1, 3, 4];
        INITIAL_DICE_TABLE[side as usize]
    }

    fn rot_comp(x: FlatDiceRot, y: FlatDiceRot) -> (u8, FlatDiceRot) {
        use FlatDiceRot::*;
        static MUL_TABLE: [[(u8, FlatDiceRot); 6]; 6] = {
            let (u, l, d, r, b, e) = (ToUp, ToLeft, ToDown, ToRight, ToBottom, ToTop);
            [
                [(0, b), (3, l), (0, e), (1, r), (0, d), (0, u)],
                [(1, u), (2, b), (3, d), (0, e), (2, r), (0, l)],
                [(0, e), (1, l), (0, b), (3, r), (0, u), (0, d)],
                [(3, u), (0, e), (1, d), (2, b), (2, l), (0, r)],
                [(0, d), (2, l), (0, u), (2, r), (0, e), (0, b)],
                [(0, u), (0, l), (0, d), (0, r), (0, d), (0, e)],
            ]
        };

        MUL_TABLE[y as usize][x as usize]
    }

    fn dir_to_rot(x: Direction) -> FlatDiceRot {
        static MAP_TABLE: [FlatDiceRot; 4] = [
            FlatDiceRot::ToUp,
            FlatDiceRot::ToLeft,
            FlatDiceRot::ToDown,
            FlatDiceRot::ToRight,
        ];

        MAP_TABLE[x as usize]
    }

    pub fn new() -> Self {
        DiceEncoding {
            ortho_rot: 0,
            flat_rot: FlatDiceRot::ToTop,
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

    pub fn apply_rotation(&mut self, d: Direction) {
        let (new_ortho_rot, new_rot) = Self::rot_comp(self.flat_rot, Self::dir_to_rot(d));
        self.ortho_rot = (self.ortho_rot + new_ortho_rot) % 4;
        self.flat_rot = new_rot;
    }

    pub fn rot_quat(&self) -> Quat {
        let flat_rot_quat = match self.flat_rot {
            FlatDiceRot::ToLeft => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            FlatDiceRot::ToRight => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            FlatDiceRot::ToUp => Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            FlatDiceRot::ToDown => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            FlatDiceRot::ToTop => Quat::IDENTITY,
            FlatDiceRot::ToBottom => Quat::from_rotation_x(std::f32::consts::PI),
        };
        let ortho_rot_quat = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2 * self.ortho_rot as f32);
        flat_rot_quat * ortho_rot_quat 
    }
}
