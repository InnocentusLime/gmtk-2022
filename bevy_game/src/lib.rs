mod level;
mod moveable;
mod states;
mod player;
mod save;
mod tile;
mod level_info;

use crate::states::setup_states;
use bevy::prelude::*;
use bevy::window::WindowDescriptor;
use bevy_inspector_egui::WorldInspectorPlugin;

use crate::moveable::MoveablePlugin;
use crate::level_info::{ LevelInfo, LevelInfoLoader };
use crate::level::LevelPlugin;
use crate::player::PlayerPlugin;

#[cfg(target_arch = "x86_64")] use bevy_framepace::FramepacePlugin;

const WINDOW_HEIGHT: f32 = 675.0f32;
const WINDOW_WIDTH: f32 = 1200.0f32;
//const ASPECT_RATIO: f32 = WINDOW_WIDTH / WINDOW_HEIGHT;

macro_rules! game_strings {
    (version) => { env!("CARGO_PKG_VERSION") };
    (game_name) => { "gluttony" };
    (title) => { concat!(game_strings!(game_name), "v. ", game_strings!(version)) };
}

pub static VERSION: &'static str = game_strings!(version);
pub static GAME_NAME: &'static str = game_strings!(game_name);
static TITLE: &'static str = game_strings!(title);

#[derive(Clone, Copy, Component)]
pub struct MenuCamera;

#[derive(Clone, Copy, Component)]
pub struct GameplayCamera;

fn window_descriptor() -> WindowDescriptor {
    WindowDescriptor {
        title: TITLE.to_owned(),
        resizable: false,
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        ..Default::default()
    }
}

pub fn create_app(log: bool, inspector: bool) -> App {
    let mut app = App::new();

    app
        .insert_resource(ClearColor(Color::hex("263238").unwrap()))
        .insert_resource(window_descriptor());

    if log {
        app.add_plugins(DefaultPlugins); 
    } else {
        app.add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<bevy::log::LogPlugin>());
    }

    if inspector { app.add_plugin(WorldInspectorPlugin::new()); }

    app
        .add_plugin(MoveablePlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(PlayerPlugin);

    #[cfg(target_arch = "x86_64")]
    app.add_plugin(FramepacePlugin::framerate(60).without_warnings());

    app
        .init_asset_loader::<LevelInfoLoader>()
        .add_asset::<LevelInfo>();
   
    app.world.spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Name::new("GameplayCamera")).insert(GameplayCamera);
    app.world.spawn()
        .insert_bundle(UiCameraBundle::default())
        .insert(Name::new("Screen Camera")).insert(MenuCamera);

    setup_states(&mut app);

    app
}