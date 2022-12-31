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

    pub fn build(commands: &mut Commands) {
        // Root node
        commands.spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
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
        });
    }
}