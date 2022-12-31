use bevy::prelude::*;

pub struct SimpleEntry {
    role: String,
    name: String,
}

pub enum SectionEntry {
    Subsection(Vec<CreditSection>),
    SimpleEntry(Vec<SimpleEntry>),
}

pub struct CreditSection {
    name: String,
    entries: SectionEntry,
}

pub struct CreditsAsset {
    sections: Vec<CreditSection>,
}

impl CreditsAsset {
    const REC_LEVEL_LIMIT: u8 = 3;

    pub fn test_values() -> Self {
        CreditsAsset {
            sections: vec![
                CreditSection {
                    name: "Test 1".to_owned(),
                    entries: SectionEntry::SimpleEntry(vec![
                        SimpleEntry {
                            role: "Impostor1".to_owned(),
                            name: "Someone's brother".to_owned(),
                        },
                        SimpleEntry {
                            role: "Impostor2".to_owned(),
                            name: "Someone's sister".to_owned(),
                        },
                    ]),
                },
                CreditSection {
                    name: "Test 2".to_owned(),
                    entries: SectionEntry::SimpleEntry(vec![
                        SimpleEntry {
                            role: "Impostor1".to_owned(),
                            name: "Someone's brother".to_owned(),
                        },
                        SimpleEntry {
                            role: "Impostor2".to_owned(),
                            name: "Someone's sister".to_owned(),
                        },
                    ]),
                },
                CreditSection {
                    name: "Test Co".to_owned(),
                    entries: SectionEntry::Subsection(vec![
                        CreditSection {
                            name: "Amazing department".to_owned(),
                            entries: SectionEntry::SimpleEntry(vec![
                                SimpleEntry {
                                    role: "Impostor1".to_owned(),
                                    name: "Someone's brother".to_owned(),
                                },
                                SimpleEntry {
                                    role: "Impostor2".to_owned(),
                                    name: "Someone's sister".to_owned(),
                                },
                            ]),
                        },
                        CreditSection {
                            name: "Cute department".to_owned(),
                            entries: SectionEntry::SimpleEntry(vec![
                                SimpleEntry {
                                    role: "Impostor1".to_owned(),
                                    name: "Someone's brother".to_owned(),
                                },
                                SimpleEntry {
                                    role: "Impostor2".to_owned(),
                                    name: "Someone's sister".to_owned(),
                                },
                            ]),
                        },
                        CreditSection {
                            name: "Cool department".to_owned(),
                            entries: SectionEntry::SimpleEntry(vec![
                                SimpleEntry {
                                    role: "Impostor1".to_owned(),
                                    name: "Someone's brother".to_owned(),
                                },
                                SimpleEntry {
                                    role: "Impostor2".to_owned(),
                                    name: "Someone's sister".to_owned(),
                                },
                            ]),
                        },
                    ]),
                },
            ],
        }
    }

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
        ))
        .with_children(|commands| {
            for section in self.sections.iter() {
                Self::build_credit_section(commands, 0, section, font);
            }
        });
    }

    fn build_simple_credits(
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
                        width: Val::Px(300f32),
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

    fn build_credit_section(
        commands: &mut ChildBuilder,
        rec_level: u8,
        section: &CreditSection,
        font: &Handle<Font>,
    ) {
        if rec_level > Self::REC_LEVEL_LIMIT {
            error!("Recursion level {rec_level} has reached the recusion limit");
            return;
        }

        let heading_size = (
            (Self::REC_LEVEL_LIMIT - rec_level) as f32 /
            Self::REC_LEVEL_LIMIT as f32
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
                            format!("· {} ·", section.name.to_uppercase())
                        } else {
                            section.name.clone()
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

            match &section.entries {
                SectionEntry::SimpleEntry(x) => {
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
                        Self::build_simple_credits(
                            commands,
                            x,
                            font,
                        )
                    });
                },
                SectionEntry::Subsection(x) => {
                    for section in x.iter() {
                        Self::build_credit_section(
                            commands,
                            rec_level + 1,
                            section,
                            font
                        )
                    }
                },
            }
        });
    }
}