use crate::tile::LEVEL_TILE_SIZE;
pub struct LaunchParams<'a> {
    pub logging: bool,
    pub editor: bool,
    pub level_file: Option<&'a str>,
}

#[allow(clippy::derivable_impls)]
impl Default for LaunchParams<'static> {
    fn default() -> Self {
        LaunchParams {
            logging: cfg!(debug_assertions),
            editor: cfg!(debug_assertions),
            level_file: None,
        }
    }
}

pub const WINDOW_HEIGHT: f32 = 600.0;
pub const WINDOW_WIDTH: f32 = 1150.0;

pub const TILE_IN_SCREEN_HORIZ: f32 = 30.0;
pub const TILE_IN_SCREEN_VERT: f32 = 15.0;
pub const WORLD_WIDTH: f32 = LEVEL_TILE_SIZE * TILE_IN_SCREEN_HORIZ;
pub const WORLD_HEIGHT: f32 = LEVEL_TILE_SIZE * TILE_IN_SCREEN_VERT;

/// A macro to generate the strings for the game
macro_rules! game_strings {
    (dev_name) => { "SeptemModi" };
    (version) => { env!("CARGO_PKG_VERSION") };
    (game_name) => { "gluttony" };
    (title) => { concat!(game_strings!(game_name), "v. ", game_strings!(version)) };
}

/// The version string of the game
pub static VERSION: &str = game_strings!(version);

/// The name of the game
pub static GAME_NAME: &str = game_strings!(game_name);

/// The title displayed on the window
pub static LAUNCHER_TITLE: &str = game_strings!(title);

/// The name of the develop
pub static DEV_NAME: &str = game_strings!(dev_name);