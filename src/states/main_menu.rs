use bevy::prelude::*;
use bevy_pkv::PkvStore;
use bevy_asset_loader::{ asset_collection::*, dynamic_asset::* };
use iyes_loopless::prelude::*;

use super::{ GameState, enter_level };
use crate::save::Save;
use crate::level_info::LevelInfo;
use crate::{GameplayCamera, LaunchParams};

#[derive(Resource, AssetCollection)]
pub struct MenuAssets {
    #[asset(path = "fonts/plain.ttf")]
    pub main_font: Handle<Font>,
    #[asset(path = "maps/level_info.level-info")]
    pub level_info: Handle<LevelInfo>,
}


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
                        flex_direction: FlexDirection::ColumnReverse,
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
                            ).with_alignment(TextAlignment::CENTER_LEFT),
                            ..default()
                        });

                    // Container for the controls
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(90.0), Val::Percent(80.0)),
                                flex_direction: FlexDirection::ColumnReverse,
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
                                                ).with_alignment(TextAlignment::CENTER),
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
    pkv: Res<PkvStore>,
) {
    info!("Entered main menu state");
    let save = pkv.get::<Save>("save").unwrap_or_else(|_| Save::new());
    spawn_text(&mut commands, &save, &menu_assets);

    commands.insert_resource(save);
}

fn tick(
    button_q: Query<(&Interaction, &MainMenuButton)>,
    mut commands: Commands,
    save: Option<Res<Save>>,
    mut asset_keys: ResMut<DynamicAssets>,
) {
    for (interaction, button) in button_q.iter() {
        if *interaction != Interaction::Clicked { break; }
        match button {
            MainMenuButton::PickLevel => {
                if let Some(save) = save.as_ref() {
                    let (world, level) = save.world_level();
                    enter_level(format!("maps/level{}-{}.tmx", world, level), &mut commands, &mut asset_keys);
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
    elems_query: Query<Entity, Without<GameplayCamera>>,
) {
    info!("Exited main menu state");

    for e in elems_query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_states(app: &mut App, _params: &LaunchParams) {
    app
        .add_enter_system(GameState::MainMenu, enter)
        .add_system(tick.run_in_state(GameState::MainMenu))
        .add_exit_system(GameState::MainMenu, exit);
}
