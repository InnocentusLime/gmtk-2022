use bevy::prelude::*;
use bevy_asset_loader::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::tasks::{ IoTaskPool, Task };
use futures_lite::future;
use iyes_loopless::prelude::*;

use super::GameState;
use super::loading::LoadingLevelSubstate;
use crate::save::Save;
use crate::level_info::LevelInfo;
use crate::{ GameplayCamera, MenuCamera };

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

#[derive(Clone, Copy, Component)]
enum MainMenuButton {
    PickLevel,
    Achievements,
    Settings,
    Quit,
}

fn spawn_text(
    commands: &mut Commands,
    save: &Save,
    menu_assets: Res<MenuAssets>,
) {
    let (world, level) = save.world_level();
    let font = menu_assets.main_font.clone();

    // Root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            // Left fill
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                });

            // Right fill
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(40.0), Val::Percent(100.0)),
                        flex_direction: FlexDirection::ColumnReverse,
                        align_items: AlignItems::Center,
                        margin: Rect {
                            top: Val::Px(5.0),
                            ..default()
                        },
                        ..default()
                    },
                    color: Color::rgba(0.0, 0.15, 0.15, 0.1).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Header
                    parent
                        .spawn_bundle(TextBundle {
                            style: Style {
                                margin: Rect{
                                    top: Val::Px(50.0),
                                    ..default()
                                },
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
                        });

                    // Container for the controls
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(90.0), Val::Percent(80.0)),
                                flex_direction: FlexDirection::ColumnReverse,
                                position: Rect {
                                    left: Val::Percent(5.0),
                                    ..default()
                                },
                                margin: Rect {
                                    top: Val::Px(30.0),
                                    ..default()
                                },
                                ..default()
                            },
                            color: Color::NONE.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            let mut spawn_button = |title, tag| {
                                parent
                                    .spawn_bundle(ButtonBundle {
                                        style: Style {
                                            align_items: AlignItems::Center,
                                            size: Size::new(Val::Percent(100.0), Val::Auto),
                                            padding: Rect {
                                                left: Val::Px(30.0),
                                                top: Val::Px(18.0),
                                                bottom: Val::Px(13.0),
                                                ..default()
                                            },
                                            margin: Rect {
                                                bottom: Val::Px(40.0),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        color: Color::rgb(0.0, 0.15, 0.15).into(),
                                        ..default()
                                    })
                                    .insert(tag)
                                    .with_children(|parent| {
                                        parent
                                            .spawn_bundle(TextBundle {
                                                style: Style {
                                                    ..default()
                                                },
                                                text: Text::with_section(
                                                    title, 
                                                    TextStyle {
                                                        font: font.clone(),
                                                        font_size: 35.0f32,
                                                        color: Color::WHITE,
                                                    },
                                                    TextAlignment {
                                                        vertical: VerticalAlign::Center,
                                                        horizontal: HorizontalAlign::Center,
                                                    }
                                                ),
                                                ..default()
                                            });
                                    });
                            };

                            spawn_button("Choose level", MainMenuButton::PickLevel);
                            spawn_button("Achievements", MainMenuButton::Achievements);
                            spawn_button("Settings", MainMenuButton::Settings);
                            spawn_button("Quit", MainMenuButton::Quit);
                        });
                });
        });
}


fn enter(
    mut commands: Commands, 
    menu_assets: Res<MenuAssets>,
    io_pool: Res<IoTaskPool>,
    save: Option<Res<Save>>,
) {
    info!("Entered main menu state");

    if let Some(save) = save { 
        spawn_text(&mut commands, &save, menu_assets);
        return; 
    }

    commands.spawn()
        .insert(SaveLoading(io_pool.spawn(async move {
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
    button_q: Query<(&Interaction, &MainMenuButton)>,
    mut commands: Commands,
    save: Option<Res<Save>>,
    mut asset_keys: ResMut<DynamicAssets>,
) {
    use bevy::input::ElementState;

    for (interaction, button) in button_q.iter() {
        if *interaction != Interaction::Clicked { break; }
        match button {
            MainMenuButton::PickLevel => {
                if let Some(save) = save.as_ref() {
                    let (world, level) = save.world_level();
                    // TODO special function, which handles things like entering the level
                    // It would verify the integrity of a save before proceeding.
                    asset_keys.register_asset(
                        "level",
                        DynamicAsset::File {
                            path: format!("maps/level{}-{}.tmx", world, level),
                        }
                    );
                    commands.insert_resource(NextState(GameState::LoadingLevel(LoadingLevelSubstate::LoadingBaseAssets)));
                }
            },
            MainMenuButton::Achievements => (),
            MainMenuButton::Settings => (),
            MainMenuButton::Quit => (),
        }
    }
}

fn exit(
    mut commands: Commands,
    elems_query: Query<Entity, (Without<GameplayCamera>, Without<MenuCamera>)>,
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
