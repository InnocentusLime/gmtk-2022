use bevy::prelude::*;

pub enum SectionEntry {
    Subsection(CreditSection),
    SimpleEntry {
        role: String,
        name: String,
    }
}

pub struct CreditSection {
    name: String,
    entries: Vec<SectionEntry>,
}

pub struct CreditsAsset {
    sections: Vec<CreditSection>,
}

impl CreditsAsset {
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