use anyhow::{bail, ensure, anyhow};
use bevy::{prelude::*, asset::{AssetLoader, LoadedAsset}};
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel};

const REC_LEVEL_LIMIT: u8 = 3;

pub struct SimpleEntry {
    role: String,
    name: String,
}

impl SimpleEntry {
    fn list_item(parser: &mut Parser) -> anyhow::Result<String> {
        // Ensure the tag opens
        ensure!(matches!(parser.next(), Some(Event::Start(Tag::Item))), "Expected the list item to end");

        let res = match parser.next() {
            Some(Event::Text(x)) => x.to_string(),
            _ => bail!("Expected a text tag"),
        };

        // Ensure the tag closes
        ensure!(matches!(parser.next(), Some(Event::End(Tag::Item))), "Expected the list item to end");

        Ok(res)
    }

    fn from_parser(parser: &mut Parser) -> anyhow::Result<Option<Self>> {
        // We need a list item -- otherwise we are expecting the list end
        match parser.next() {
            Some(Event::Start(Tag::Item)) => (),
            Some(Event::End(Tag::List(None))) => return Ok(None),
            _ => bail!("Expected either an item start or the list end"),
        }

        // A credit entry is a list
        ensure!(matches!(parser.next(), Some(Event::Start(Tag::List(None)))), "The item must contain an unenumerated list");

        // Get role and name
        let role = Self::list_item(parser)?;
        let name = Self::list_item(parser)?;

        // A credit entry is a list
        ensure!(matches!(parser.next(), Some(Event::End(Tag::List(None)))), "Expected the list to end");

        // Ensure the tag closes
        ensure!(matches!(parser.next(), Some(Event::End(Tag::Item))), "Expected the list item to end");

        Ok(Some(Self { role, name }))
    }

    fn from_parser_vec(parser: &mut Parser) -> anyhow::Result<Vec<Self>> {
        std::iter::from_fn(|| Self::from_parser(parser).transpose()).collect()
    }
}

pub enum SectionEntry {
    Subsection(Vec<CreditSection>),
    SimpleEntry(Vec<SimpleEntry>),
}

pub struct CreditSection {
    name: String,
    entries: SectionEntry,
}

impl CreditSection {
    fn next_level(level: HeadingLevel) -> Option<HeadingLevel> {
        match level {
            HeadingLevel::H1 => Some(HeadingLevel::H2),
            HeadingLevel::H2 => Some(HeadingLevel::H3),
            HeadingLevel::H3 => Some(HeadingLevel::H4),
            HeadingLevel::H4 => Some(HeadingLevel::H5),
            HeadingLevel::H5 => Some(HeadingLevel::H6),
            HeadingLevel::H6 => None,
        }
    }

    fn level_depth(level: HeadingLevel) -> u8 {
        match level {
            HeadingLevel::H1 => 0,
            HeadingLevel::H2 => 1,
            HeadingLevel::H3 => 2,
            HeadingLevel::H4 => 3,
            HeadingLevel::H5 => 4,
            HeadingLevel::H6 => 5,
        }
    }

    fn err_child_no_title() -> anyhow::Error { anyhow!("Expected the child to have a title") }

    fn from_parser_child(
        parser: &mut Parser,
        parent_level: HeadingLevel,
        parent_title: &str,
        heading: Option<(HeadingLevel, String)>,
    ) -> anyhow::Result<Option<Self>> {
        let (level, name) = match heading {
            Some(x) => x,
            None => match parser.next() {
                None => return Ok(None),
                Some(Event::Start(Tag::Heading(level, name, _))) => (
                    level,
                    name.ok_or_else(Self::err_child_no_title)?.to_owned(),
                ),
                Some(ref e @ Event::End(Tag::Heading(level, name, _))) => {
                    ensure!(parent_level != level && Some(parent_title) != name, "Unexpected end tag {e:?}");
                    return Ok(None);
                },
                _ => bail!("Expected either parent end or start of a section"),
            }
        };

        Ok(Some(Self::from_parser_rec(parser, level, name)?))
    }

    // Since the parser reads the start of the first entry, they need
    // to be provided to this procedure.
    fn from_parser_vec(
        parser: &mut Parser,
        parent_level: HeadingLevel,
        parent_title: &str,
        level: HeadingLevel,
        name: String,
    ) -> anyhow::Result<Vec<Self>> {
        ensure!(Some(level) == Self::next_level(parent_level), "Child level must be parent level + 1 ({parent_level} + 1)");
        let mut state = Some((level, name));

        std::iter::from_fn(|| Self::from_parser_child(
            parser,
            parent_level,
            parent_title,
            state.take()
        ).transpose())
        .collect()
    }

    fn from_parser_rec(
        parser: &mut Parser,
        level: HeadingLevel,
        name: String,
    ) -> anyhow::Result<Self> {
        // Check the depth level
        let expected_level = Self::level_depth(level);
        if Self::level_depth(level) >= REC_LEVEL_LIMIT {
            bail!("Recursion depth limit reached (level {expected_level}, while max is {REC_LEVEL_LIMIT})");
        }

        // Read the body of the section
        let entries = match parser.next() {
            None => bail!("Expected a list start or the heading start"),
            Some(Event::Start(Tag::List(None))) => SectionEntry::SimpleEntry(SimpleEntry::from_parser_vec(parser)?),
            Some(Event::Start(Tag::Heading(child_level, child_name, _))) => SectionEntry::Subsection(
                CreditSection::from_parser_vec(
                    parser,
                    level,
                    name.as_str(),
                    child_level,
                    child_name.ok_or_else(Self::err_child_no_title)?.to_owned(),
                )?
            ),
            Some(e) => bail!("Unexpected event: {e:?}"),
        };

        // Read the end of the section
        match parser.next() {
            Some(Event::End(Tag::Heading(the_level, the_name, _)))
                if the_level == level && the_name == Some(&name) => (),
            _ => bail!("Expected closing tag for {name} on level {level}"),
        }

        Ok(Self {
            name,
            entries,
        })
    }
}

#[derive(bevy::reflect::TypeUuid)]
#[uuid = "08e38c4f-9bc2-4901-98f0-5be11451b17f"]
pub struct CreditsAsset {
    sections: Vec<CreditSection>,
}

impl CreditsAsset {
    pub fn test_values() -> Self {
        CreditsAsset {
            sections: vec![
                CreditSection {
                    name: "Game writing".to_owned(),
                    entries: SectionEntry::SimpleEntry(vec![
                        SimpleEntry {
                            role: "Making puter beep".to_owned(),
                            name: "Meeeeee".to_owned(),
                        },
                        SimpleEntry {
                            role: "Magic".to_owned(),
                            name: "Susie the witch".to_owned(),
                        },
                    ]),
                },
                CreditSection {
                    name: "Some business".to_owned(),
                    entries: SectionEntry::SimpleEntry(vec![
                        SimpleEntry {
                            role: "Dollar managing".to_owned(),
                            name: "Fred Fuqqs".to_owned(),
                        },
                        SimpleEntry {
                            role: "Merketting of the product".to_owned(),
                            name: "Classical Frodo".to_owned(),
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

    fn from_parser_section(parser: &mut Parser) -> anyhow::Result<Option<CreditSection>> {
        let (level, name) = match parser.next() {
            None => return Ok(None),
            Some(Event::Start(Tag::Heading(level, name, _))) => (level, name),
            _ => bail!("Expected a start of a heading or end of stream"),
        };
        let name = name.ok_or_else(|| anyhow!("Section must have a title"))?.to_owned();
        ensure!(level == HeadingLevel::H1, "Expected level 1");
        Ok(Some(CreditSection::from_parser_rec(parser, level, name)?))
    }

    fn from_parser(mut parser: Parser) -> anyhow::Result<Self> {
        let sections =
            std::iter::from_fn(|| Self::from_parser_section(&mut parser).transpose())
            .collect::<Result<_, _>>()?;

        Ok(CreditsAsset { sections })
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

    fn build_credit_section(
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

#[derive(Default, Clone, Copy)]
pub struct CreditsAssetLoader;

impl AssetLoader for CreditsAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let asset =  CreditsAsset::from_parser(
                Parser::new(std::str::from_utf8(bytes)?)
            )?;
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] { &["cds"] }
}