use bevy::prelude::*;
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

#[derive(Clone, Copy, Component)]
struct WaitingSaveTag;

#[derive(Component)]
struct SaveLoading(Task<Save>);

fn spawn_text(
    commands: &mut Commands,
    save: &Save,
    menu_assets: Res<MenuAssets>,
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
    save: Option<Res<Save>>,
) {
    info!("Entered main menu state");

    if let Some(save) = save { 
        spawn_text(&mut commands, &save, menu_assets);
        return; 
    }

    commands.spawn()
        .insert(SaveLoading(IoTaskPool::get().spawn(async move {
            match Save::load() {
                Ok(x) => x,
                Err(e) => {
                    warn!("Error loading save: {}\nReset save will be used.", e);
                    let res = Save::new();
                    if let Err(e) = res.save() {
                        error!("Failed to save the new save: {}\nAny progress will be lost.", e);
                    }
                    res
                },
            }
        })));


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
            "Reading save...",
            TextStyle {
                font: font.clone(),
                font_size: 30.0f32,
                color: Color::WHITE,
            },
        ).with_alignment(TextAlignment::CENTER),
        ..default()
    }).insert(WaitingSaveTag);
}

fn save_await(
    mut commands: Commands,
    menu_assets: Res<MenuAssets>,
    mut q: Query<(Entity, &mut SaveLoading)>,
    waiting: Query<Entity, With<WaitingSaveTag>>,
) {
    match q.get_single_mut() {
        Ok((e, mut task)) => if let Some(save) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(e).despawn();
            for e in waiting.iter() {
                commands.entity(e).despawn();
            }
            spawn_text(&mut commands, &save, menu_assets);
            commands.insert_resource(save);
        }
        Err(_) => (),
    }
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
        .add_system(save_await.run_in_state(GameState::MainMenu))
        .add_system(tick.run_in_state(GameState::MainMenu))
        .add_exit_system(GameState::MainMenu, exit);
}
