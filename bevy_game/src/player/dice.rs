#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Direction {
    Up = 0,
    Left = 1,
    Down = 2,
    Right = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct DiceEncoding {
    horiz: [u8; 4],
    vert: [u8; 4],
}

impl DiceEncoding {
    pub fn new() -> Self {
        DiceEncoding {
            horiz: [5, 1, 2, 6],
            vert: [4, 2, 3, 5],
        }
    }

    fn shift_array_front(arr: &mut [u8; 4]) {
        let last = arr[3];
        arr[3] = arr[2];
        arr[2] = arr[1];
        arr[1] = arr[0];
        arr[0] = last;
    }
    
    fn shift_array_back(arr: &mut [u8; 4]) {
        let first = arr[0];
        arr[0] = arr[1];
        arr[1] = arr[2];
        arr[2] = arr[3];
        arr[3] = first;
    }

    fn shift_horiz_front(&mut self) {
        Self::shift_array_front(&mut self.horiz);
        self.vert[1] = self.horiz[2];
        self.vert[3] = self.horiz[0];
    }
    
    fn shift_horiz_back(&mut self) {
        Self::shift_array_back(&mut self.horiz);
        self.vert[1] = self.horiz[2];
        self.vert[3] = self.horiz[0];
    }

    fn shift_vert_front(&mut self) {
        Self::shift_array_front(&mut self.vert);
        self.horiz[2] = self.vert[1];
        self.horiz[0] = self.vert[3];
    }
    
    fn shift_vert_back(&mut self) {
        Self::shift_array_back(&mut self.vert);
        self.horiz[2] = self.vert[1];
        self.horiz[0] = self.vert[3];
    }

    pub fn upper_side(self) -> u8 {
        self.horiz[2]
    }

    pub fn apply_rotation(&mut self, dir: Direction) {
        match dir {
            Direction::Up => self.shift_vert_front(),
            Direction::Left => self.shift_horiz_back(),
            Direction::Down => self.shift_vert_back(),
            Direction::Right => self.shift_horiz_front(),
        }
    }
}
