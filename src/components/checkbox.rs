use bevy::{
    prelude::*,
    ui::Checked,
    ui_widgets::{Checkbox, checkbox_self_update, observe},
};

use crate::components::{HOVERED, PRESSED};

const CHECKBOX_OUTLINE: Color = Color::srgb(0.45, 0.45, 0.45);
const CHECKBOX_CHECK: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn checkbox() -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            height: percent(100),
            aspect_ratio: Some(1.0),
            border: UiRect::all(px(4)),
            ..default()
        },
        BorderColor::all(CHECKBOX_OUTLINE),
        BorderRadius::all(px(4)),
        Interaction::default(),
        Checkbox,
        observe(checkbox_self_update),
    )
}

pub(super) fn update_checkbox_style(
    mut checkbox_q: Query<
        (
            &Interaction,
            &mut BorderColor,
            Has<Checked>,
            &mut BackgroundColor,
        ),
        With<Checkbox>,
    >,
) {
    for (interaction, mut border_color, checked, mut background_color) in checkbox_q.iter_mut() {
        border_color.set_all(match *interaction {
            Interaction::Pressed => CHECKBOX_OUTLINE.lighter(PRESSED),
            Interaction::Hovered => CHECKBOX_OUTLINE.lighter(HOVERED),
            Interaction::None => CHECKBOX_OUTLINE,
        });

        *background_color = BackgroundColor(match checked {
            true => CHECKBOX_CHECK,
            false => Srgba::NONE.into(),
        });
    }
}
