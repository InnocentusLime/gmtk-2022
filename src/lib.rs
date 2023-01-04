mod level;
mod moveable;
mod states;
mod player;
mod save;
mod tile;
mod level_info;
mod config;

use bevy::render::camera::{WindowOrigin, ScalingMode};
use states::setup_states;
use bevy::{prelude::*};
use bevy::window::WindowDescriptor;
use bevy_pkv::PkvStore;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_common_assets::json::JsonAssetPlugin;

use moveable::MoveablePlugin;
use level_info::LevelInfo;
use level::LevelPlugin;
use player::PlayerPlugin;

pub use config::*;

#[cfg(target_arch = "x86_64")] use bevy_framepace::{ FramepacePlugin, FramepaceSettings, Limiter };

#[derive(Clone, Copy, Component)]
pub struct GameplayCamera;

fn window_descriptor() -> WindowDescriptor {
    WindowDescriptor {
        title: LAUNCHER_TITLE.to_owned(),
        resizable: false,
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        canvas: Some("#bevy".to_string()),
        fit_canvas_to_parent: true,
        ..Default::default()
    }
}

pub fn app(params: LaunchParams) -> App {
    let mut app = App::new();

    // Insert base resources
    app
        .insert_resource(PkvStore::new(DEV_NAME, GAME_NAME))
        .insert_resource(ClearColor(Color::hex("263238").unwrap()));

    // Load bevy's core
    DefaultPlugins
        .set(WindowPlugin {
            window: window_descriptor(),
            ..default()
        })
        .finish(&mut app);

    // Load framepace
    app
        .insert_resource(FramepaceSettings::default().with_limiter(Limiter::from_framerate(60.0)))
        .add_plugin(FramepacePlugin);

    // Init or not init inspector (DO IT BEFORE THE GAME PLUGINS)
    if params.inspector {
        app.add_plugin(WorldInspectorPlugin::new());
    }

    // Game plugins
    app
        .add_plugin(JsonAssetPlugin::<LevelInfo>::new(&["level-info"]))
        .add_plugin(MoveablePlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(PlayerPlugin);


    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection = OrthographicProjection {
        left: -300f32,
        right: 300f32,
        top: 300f32,
        bottom: -300f32,
        far: 1000f32,
        near: 0.0f32,
        window_origin: WindowOrigin::Center,
        scaling_mode: ScalingMode::FixedHorizontal(600f32),
        scale: 1.0f32,
    };
    app.world.spawn((
        camera_bundle,
        Name::new("GameplayCamera"),
        GameplayCamera,
    ));

    setup_states(&mut app, &params);

    app
}
