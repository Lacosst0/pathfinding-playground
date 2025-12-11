use bevy::{prelude::*, ui_widgets::Button};

use crate::components::{HOVERED, PRESSED};

const BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);

pub fn button(text: impl Bundle) -> impl Bundle {
    (
        Node {
            border: Val::all(px(2)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: Val::all(px(4.)),
            ..default()
        },
        Button,
        Interaction::default(),
        BorderColor::all(Color::BLACK),
        BorderRadius::all(percent(25.)),
        BackgroundColor(BUTTON),
        children![(text, TextColor(Color::srgb(0.9, 0.9, 0.9)),)],
    )
}

pub(super) fn update_button_style(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_q {
        *color = (match *interaction {
            Interaction::Pressed => BUTTON.lighter(PRESSED),
            Interaction::Hovered => BUTTON.lighter(HOVERED),
            Interaction::None => BUTTON,
        })
        .into();
    }
}
