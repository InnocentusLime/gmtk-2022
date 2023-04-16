use bevy::prelude::*;
use bevy_pkv::PkvStore;
use bevy_asset_loader::{ asset_collection::*, dynamic_asset::* };

use super::loading::LoadingLevel;
use super::{ GameState, enter_level };
use crate::save::Save;
use crate::level_info::LevelInfo;
use crate::LaunchParams;

#[derive(Resource, AssetCollection)]
pub struct MenuAssets {
    #[asset(path = "fonts/plain.ttf")]
    pub main_font: Handle<Font>,
    #[asset(path = "maps/level_info.level-info")]
    pub level_info: Handle<LevelInfo>,
}

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct MainMenuTag;

#[derive(Clone, Copy, Component)]
enum MainMenuButton {
    PickLevel,
    Achievements,
    Settings,
    Quit,
}

fn spawn_menu(
    commands: &mut Commands,
    save: &Save,
    menu_assets: &MenuAssets,
) {
    let (_world, _level) = save.world_level();
    let font = menu_assets.main_font.clone();

    // Root node
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()

            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            // Left fill
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                });

            // Right fill
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(40.0), Val::Percent(100.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect {
                            top: Val::Px(5.0),
                            ..default()
                        },
                        ..default()
                    },
                    background_color: Color::rgba(0.0, 0.15, 0.15, 0.1).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Header
                    parent
                        .spawn(TextBundle {
                            style: Style {
                                margin: UiRect{
                                    top: Val::Px(50.0),
                                    ..default()
                                },
                                ..default()
                            },
                            text: Text::from_section(
                                "Main menu",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 60.0f32,
                                    color: Color::WHITE,
                                },
                            ).with_alignment(TextAlignment::Center),
                            ..default()
                        });

                    // Container for the controls
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(90.0), Val::Percent(80.0)),
                                flex_direction: FlexDirection::Column,
                                position: UiRect {
                                    left: Val::Percent(5.0),
                                    ..default()
                                },
                                margin: UiRect {
                                    top: Val::Px(30.0),
                                    ..default()
                                },
                                ..default()
                            },
                            background_color: Color::NONE.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            let mut spawn_button = |title, tag| {
                                parent
                                    .spawn(ButtonBundle {
                                        style: Style {
                                            align_items: AlignItems::Center,
                                            size: Size::new(Val::Percent(100.0), Val::Auto),
                                            padding: UiRect {
                                                left: Val::Px(30.0),
                                                top: Val::Px(18.0),
                                                bottom: Val::Px(13.0),
                                                ..default()
                                            },
                                            margin: UiRect {
                                                bottom: Val::Px(40.0),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        background_color: Color::rgb(0.0, 0.15, 0.15).into(),
                                        ..default()
                                    })
                                    .insert(tag)
                                    .with_children(|parent| {
                                        parent
                                            .spawn(TextBundle {
                                                style: Style {
                                                    ..default()
                                                },
                                                text: Text::from_section(
                                                    title,
                                                    TextStyle {
                                                        font: font.clone(),
                                                        font_size: 35.0f32,
                                                        color: Color::WHITE,
                                                    },
                                                ).with_alignment(TextAlignment::Center),
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
        })
        .insert(MainMenuTag);
}

fn enter(
    mut commands: Commands,
    menu_assets: Res<MenuAssets>,
    pkv: Res<PkvStore>,
) {
    info!("Entered main menu state");
    let save = pkv.get::<Save>("save").unwrap_or_else(|_| Save::new());
    spawn_menu(&mut commands, &save, &menu_assets);

    commands.insert_resource(save);
}

fn tick(
    button_q: Query<(&Interaction, &MainMenuButton)>,
    save: Option<Res<Save>>,
    mut asset_keys: ResMut<DynamicAssets>,
    mut load_lvl_st: ResMut<NextState<LoadingLevel>>,
    mut game_st: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in button_q.iter() {
        if *interaction != Interaction::Clicked { break; }
        match button {
            MainMenuButton::PickLevel => {
                if let Some(save) = save.as_ref() {
                    let (world, level) = save.world_level();
                    enter_level(
                        format!("maps/level{}-{}.tmx", world, level),
                        &mut asset_keys,
                        &mut load_lvl_st,
                        &mut game_st,
                    );
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
    main_menu_q: Query<Entity, With<MainMenuTag>>,
) {
    info!("Exited main menu state");

    main_menu_q.for_each(|e| commands.entity(e).despawn_recursive());
}

pub fn setup_states(app: &mut App, _params: &LaunchParams) {
    app
        .add_system(enter.in_schedule(OnEnter(GameState::MainMenu)))
        .add_system(tick.run_if(in_state(GameState::MainMenu)))
        .add_system(exit.in_schedule(OnExit(GameState::MainMenu)));
}
