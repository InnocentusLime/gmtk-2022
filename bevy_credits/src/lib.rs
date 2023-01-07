use bevy::prelude::*;
use bevy_common_assets::yaml::YamlAssetPlugin;

const REC_LEVEL_LIMIT: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SimpleEntry {
    role: String,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum SectionEntry {
    Subsections(Vec<CreditSection>),
    RoleList(Vec<SimpleEntry>),
    Verbatim(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CreditSection {
    title: String,
    content: SectionEntry,
}

#[derive(bevy::reflect::TypeUuid)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[uuid = "08e38c4f-9bc2-4901-98f0-5be11451b17f"]
pub struct CreditsAsset {
    content: Vec<CreditSection>,
}

impl CreditsAsset {
    pub fn build(
        &self,
        commands: &mut Commands,
        font: &Handle<Font>,
    ) {
        // Root node
        commands
        .spawn((
            NodeBundle {
                style: Style {
                    position: UiRect {
                        top: Val::Percent(100f32),
                        ..default()
                    },
                    size: Size::new(Val::Percent(100.0), Val::Auto),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Baseline,
                    margin: UiRect {
                        top: Val::Px(5.0),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            Name::new("Credits"),
            CreditTag,
        ))
        .with_children(|commands| {
            for cont in self.content.iter() {
                Self::build_content_elem(commands, 0, cont, font);
            }
        });
    }

    fn build_role_list(
        commands: &mut ChildBuilder,
        simple: &[SimpleEntry],
        font: &Handle<Font>,
    ) {
        commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect {
                        left: Val::Percent(0.10),
                        ..default()
                    },
                    size: Size {
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            Name::new("Role column"),
        ))
        .with_children(|commands| {
            for x in simple.iter() {
                commands.spawn((
                    TextBundle {
                        style: Style {
                            align_self: AlignSelf::FlexEnd,
                            margin: UiRect{
                                top: Val::Px(5.0),
                                ..default()
                            },
                            ..default()
                        },
                        text: Text::from_section(
                            &x.role,
                            TextStyle {
                                font: font.clone(),
                                font_size: 20.0f32,
                                color: Color::WHITE,
                            },
                        ).with_alignment(TextAlignment::CENTER_RIGHT),
                        ..default()
                    },
                    Name::new("Role")
                ));
            }
        });

        commands.spawn((
            NodeBundle {
                style: Style {
                    size: Size {
                        height: Val::Px(25.0f32 * simple.len() as f32),
                        width: Val::Px(50f32),
                    },
                    ..default()
                },
                ..default()
            },
            Name::new("Separator column"),
        ));

        commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect {
                        right: Val::Percent(0.10),
                        ..default()
                    },
                    size: Size {
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            Name::new("Name column"),
        ))
        .with_children(|commands| {
            for x in simple.iter() {
                commands.spawn((
                    TextBundle {
                        style: Style {
                            align_self: AlignSelf::FlexStart,
                            margin: UiRect{
                                top: Val::Px(5.0),
                                ..default()
                            },
                            ..default()
                        },
                        text: Text::from_section(
                            &x.name,
                            TextStyle {
                                font: font.clone(),
                                font_size: 20.0f32,
                                color: Color::WHITE,
                            },
                        ).with_alignment(TextAlignment::CENTER_LEFT),
                        ..default()
                    },
                    Name::new("Name")
                ));
            }
        });
    }

    fn build_content_elem(
        commands: &mut ChildBuilder,
        rec_level: u8,
        section: &CreditSection,
        font: &Handle<Font>,
    ) {
        if rec_level > REC_LEVEL_LIMIT {
            error!("Recursion level {rec_level} has reached the recusion limit");
            return;
        }

        let heading_size = (
            (REC_LEVEL_LIMIT - rec_level) as f32 /
            REC_LEVEL_LIMIT as f32
        ).clamp(0.25f32, 1.0f32);

        commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Auto),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect {
                        top: Val::Px(5.0),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            Name::new("Section")
        ))
        .with_children(|commands| {
            commands.spawn((
                TextBundle {
                    style: Style {
                        margin: UiRect{
                            top: Val::Px(50.0),
                            ..default()
                        },
                        ..default()
                    },
                    text: Text::from_section(
                        if rec_level == 0 {
                            format!("· {} ·", section.title.to_uppercase())
                        } else {
                            section.title.clone()
                        },
                        TextStyle {
                            font: font.clone(),
                            font_size: 50.0f32 * heading_size.powf(0.5f32),
                            color: Color::WHITE,
                        },
                    ).with_alignment(TextAlignment::CENTER),
                    ..default()
                },
                Name::new("Section title"),
            ));

            match &section.content {
                SectionEntry::RoleList(x) => {
                    commands.spawn((
                        NodeBundle {
                            style: Style {
                                size: Size {
                                    width: Val::Percent(100.0f32),
                                    height: Val::Auto,
                                },
                                flex_direction: FlexDirection::Row,
                                ..default()
                            },
                            ..default()
                        },
                        Name::new("Simple credits"),
                    )).with_children(|commands| {
                        Self::build_role_list(
                            commands,
                            x,
                            font,
                        )
                    });
                },
                SectionEntry::Subsections(x) => {
                    for section in x.iter() {
                        Self::build_content_elem(
                            commands,
                            rec_level + 1,
                            section,
                            font
                        )
                    }
                },
                SectionEntry::Verbatim(x) => {
                    commands.spawn((
                        TextBundle {
                            style: Style {
                                margin: UiRect{
                                    top: Val::Px(50.0),
                                    ..default()
                                },
                                ..default()
                            },
                            text: Text::from_section(
                                x.to_owned(),
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0f32,
                                    color: Color::WHITE,
                                },
                            ).with_alignment(TextAlignment::CENTER),
                            ..default()
                        },
                        Name::new("Verbatim"),
                    ));
                },
            }
        });
    }
}

#[derive(Clone, Copy, Default, Component)]
pub struct CreditTag;

pub fn scroll_system(
    time: Res<Time>,
    mut credit_query: Query<&mut Style, With<CreditTag>>,
) {
    credit_query.for_each_mut(|mut style| {
        match &mut style.position.top {
            Val::Percent(x) => *x -= 3.0f32 * time.delta_seconds(),
            _ => (),
        }
    })
}

pub struct CreditsPlugin;

impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(scroll_system)
            .add_plugin(YamlAssetPlugin::<CreditsAsset>::new(&["cds"]));
    }
}