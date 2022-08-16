use bevy::prelude::*;
use bevy_pkv::PkvStore;
use bevy_asset_loader::{ asset_collection::*, dynamic_asset::* };
use bevy::input::keyboard::KeyboardInput;
use bevy::tasks::{ IoTaskPool, Task };
use futures_lite::future;
use iyes_loopless::prelude::*;

use super::{ GameState, enter_level };
use crate::save::Save;
use crate::level_info::LevelInfo;
use crate::GameplayCamera;

#[derive(AssetCollection)]
pub struct MenuAssets {
    #[asset(path = "fonts/plain.ttf")]
    pub main_font: Handle<Font>,
    #[asset(path = "maps/level_info.level-info")]
    pub level_info: Handle<LevelInfo>,
}

fn spawn_text(
    commands: &mut Commands,
    save: &Save,
    menu_assets: &MenuAssets,
) {
    let (world, level) = save.world_level();
    let font = menu_assets.main_font.clone();
    commands.spawn_bundle(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect { 
                left: Val::Px(500.0),
                bottom: Val::Px(340.0),
                ..default() 
            },
            ..default()
        },
        text: Text::from_section(
            format!("Press enter to enter {}-{}", world, level),
            TextStyle {
                font: font.clone(),
                font_size: 30.0f32,
                color: Color::WHITE,
            },
        ).with_alignment(TextAlignment::CENTER),
        ..default()
    });
}


fn enter(
    mut commands: Commands, 
    menu_assets: Res<MenuAssets>,
    pkv: Res<PkvStore>,
) {
    info!("Entered main menu state");

    let save = pkv.get::<Save>("save").unwrap_or_else(|_| Save::new());
    spawn_text(&mut commands, &save, &*menu_assets);

    let font = menu_assets.main_font.clone();
    commands.spawn_bundle(TextBundle {
        style: Style {
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        text: Text::from_section(
            "Main menu", 
            TextStyle {
                font: font.clone(),
                font_size: 60.0f32,
                color: Color::WHITE,
            },
        ).with_alignment(TextAlignment::CENTER),
        ..default()
    });
    
    commands.insert_resource(save);
}

fn tick(
    mut commands: Commands,
    mut events: EventReader<KeyboardInput>,
    save: Option<Res<Save>>,
    mut asset_keys: ResMut<DynamicAssets>,
) {
    use bevy::input::ButtonState;

    for ev in events.iter() {
        if ev.state == ButtonState::Pressed && ev.key_code == Some(KeyCode::Return) {
            if let Some(save) = save.as_ref() {
                let (world, level) = save.world_level();
                enter_level(format!("maps/level{}-{}.tmx", world, level), &mut commands, &mut *asset_keys);
            }
        }
    }
}

fn exit(
    mut commands: Commands,
    elems_query: Query<Entity, Without<GameplayCamera>>,
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
