use bevy::prelude::*;
use bevy::window::WindowDescriptor;
#[cfg(feature = "debugging")]
use bevy_inspector_egui::WorldInspectorPlugin;

use crate::moveable::MoveablePlugin;
use crate::level_info::{ LevelInfo, LevelInfoLoader };
use crate::level::LevelPlugin;
use crate::player::PlayerPlugin;

#[cfg(target_arch = "x86_64")] use bevy_framepace::FramepacePlugin;

const WINDOW_HEIGHT: f32 = 675.0f32;
const WINDOW_WIDTH: f32 = 1200.0f32;
//const ASPECT_RATIO: f32 = WINDOW_WIDTH / WINDOW_HEIGHT;

static TITLE : &'static str = concat!("gluttony", " v. ", env!("CARGO_PKG_VERSION"));

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

fn setup_plugins(app: &mut App) {
    app
        .insert_resource(ClearColor(Color::hex("263238").unwrap()))
        .insert_resource(window_descriptor());

    #[cfg(feature = "debugging")]
    app.add_plugins(DefaultPlugins);
    
    #[cfg(not(feature = "debugging"))]
    app.add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<bevy::log::LogPlugin>());

    app
        .add_plugin(MoveablePlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(PlayerPlugin);

    #[cfg(target_arch = "x86_64")]
    app.add_plugin(FramepacePlugin::framerate(60).without_warnings());
    
    #[cfg(feature = "debugging")]
    app.add_plugin(WorldInspectorPlugin::new());

    app
        .init_asset_loader::<LevelInfoLoader>()
        .add_asset::<LevelInfo>();
}

pub fn create_app() -> App {
    let mut app = App::new();

    setup_plugins(&mut app);
   
    app.world.spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(Name::new("GameplayCamera")).insert(GameplayCamera);
    app.world.spawn()
        .insert_bundle(UiCameraBundle::default())
        .insert(Name::new("Screen Camera")).insert(MenuCamera);

    app
}
