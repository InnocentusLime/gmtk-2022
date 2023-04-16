mod level;
mod moveable;
mod states;
mod player;
mod save;
mod tile;
mod level_info;
mod config;

use bevy::render::camera::ScalingMode;
use bevy_editor_pls::EditorPlugin;
use states::setup_states;
use bevy::{prelude::*};
use bevy::window::{Window, WindowResolution};
use bevy_pkv::PkvStore;
use bevy_common_assets::json::JsonAssetPlugin;

use moveable::MoveablePlugin;
use level_info::LevelInfo;
use level::LevelPlugin;
use player::PlayerPlugin;

pub use config::*;

#[cfg(target_arch = "x86_64")] use bevy_framepace::{ FramepacePlugin, FramepaceSettings, Limiter };

#[derive(Clone, Copy, Component)]
pub struct GameplayCamera;

fn game_window() -> Window {
    Window {
        title: LAUNCHER_TITLE.to_owned(),
        resizable: false,
        resolution: WindowResolution::new(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        ),
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
            primary_window: Some(game_window()),
            ..default()
        })
        .finish(&mut app);

    // Load framepace
    app
        .insert_resource(FramepaceSettings::default().with_limiter(Limiter::from_framerate(60.0)))
        .add_plugin(FramepacePlugin);

    // Init or not init inspector (DO IT BEFORE THE GAME PLUGINS)
    if params.editor {
        app.add_plugin(EditorPlugin::default());
    }

    // Game plugins
    app
        .add_plugin(JsonAssetPlugin::<LevelInfo>::new(&["level-info"]))
        .add_plugin(MoveablePlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(PlayerPlugin);


    let camera_bundle = Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: WORLD_WIDTH,
                height: WORLD_HEIGHT,
            },
            ..default()
        },
        ..default()
    };
    app.world.spawn((
        camera_bundle,
        Name::new("GameplayCamera"),
        GameplayCamera,
    ));

    setup_states(&mut app, &params);

    app
}
