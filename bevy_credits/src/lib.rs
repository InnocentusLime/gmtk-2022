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
        commands.spawn(NodeBundle {
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
            color: Color::rgba(0.0, 0.15, 0.15, 0.1).into(),
            ..default()
        });
    }
}