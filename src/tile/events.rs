#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileEvent {
    ExitReached,
    ButtonPressed { button_id: u8 },
}