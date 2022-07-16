use bevy::prelude::*;
use bevy_asset_loader::*;
use bevy::input::keyboard::KeyboardInput;
use iyes_loopless::prelude::*;

use super::GameState;

#[derive(AssetCollection)]
pub struct MenuAssets {
    #[asset(path = "fonts/plain.ttf")]
    main_font: Handle<Font>,
}

#[derive(Clone, Copy, Component)]
struct MenuTag;

fn enter(mut commands: Commands, menu_assets: Res<MenuAssets>) {
    info!("Entered main menu state");

    let font = menu_assets.main_font.clone();
    commands.spawn_bundle(TextBundle {
        style: Style {
            margin: Rect::all(Val::Px(5.0)),
            ..default()
        },
        text: Text::with_section(
            "Main menu", 
            TextStyle {
                font: font.clone(),
                font_size: 60.0f32,
                color: Color::WHITE,
            },
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            }
        ),
        ..default()
    }).insert(MenuTag);
    
    commands.spawn_bundle(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect { 
                left: Val::Px(550.0),
                bottom: Val::Px(340.0),
                ..default() 
            },
            ..default()
        },
        text: Text::with_section(
            "Press enter", 
            TextStyle {
                font: font.clone(),
                font_size: 30.0f32,
                color: Color::WHITE,
            },
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            }
        ),
        ..default()
    }).insert(MenuTag);
}

fn tick(
    mut commands: Commands,
    mut events: EventReader<KeyboardInput>,
) {
    use bevy::input::ElementState;

    for ev in events.iter() {
        if ev.state == ElementState::Pressed && ev.key_code == Some(KeyCode::Return) {
            commands.insert_resource(NextState(GameState::Loading));
        }
    }
}

fn exit(
    mut commands: Commands,
    elems_query: Query<Entity, With<MenuTag>>,
) {
    info!("Exited main menu state");

    for e in elems_query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_states(app: &mut App) {
    app
        .add_enter_system(GameState::MainMenu, enter)
        .add_system(tick.run_in_state(GameState::MainMenu))
        .add_exit_system(GameState::MainMenu, exit);
}
