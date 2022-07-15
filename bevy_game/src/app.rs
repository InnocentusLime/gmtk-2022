use bevy::prelude::*;
use bevy::window::WindowDescriptor;
//use bevy_framepace::FramepacePlugin;
use bevy_inspector_egui::WorldInspectorPlugin;

const WINDOW_HEIGHT : f32 = 675.0f32;
const WINDOW_WIDTH : f32 = 1200.0f32;
//const ASPECT_RATIO : f32 = WINDOW_WIDTH / WINDOW_HEIGHT;

static TITLE : &'static str = concat!("jam entry", " v. ", env!("CARGO_PKG_VERSION"));

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
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .insert_resource(window_descriptor())
        .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "x86_64")]
    app.add_plugin(FramepacePlugin::framerate(60).without_warnings());
    
    // TODO if debug
    app.add_plugin(WorldInspectorPlugin::new());
}

pub fn create_app() -> App {
    use bevy::render::camera::ScalingMode;
    
    let mut app = App::new();

    setup_plugins(&mut app);
   
    app.world.spawn()
        .insert_bundle(OrthographicCameraBundle {
            orthographic_projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical,
                ..OrthographicCameraBundle::new_2d().orthographic_projection
            },
            ..OrthographicCameraBundle::new_2d()
        }).insert(Name::new("GameplayCamera")).insert(GameplayCamera);
    app.world.spawn()
        .insert_bundle(UiCameraBundle::default())
        .insert(Name::new("Screen Camera")).insert(MenuCamera);

    app
}
