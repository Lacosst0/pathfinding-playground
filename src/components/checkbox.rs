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
    checkbox_q: Query<(Entity, &Interaction, Has<Checked>), With<Checkbox>>,
    mut borders: Query<&mut BorderColor>,
    mut marks: Query<&mut BackgroundColor>,
) {
    for (entity, interaction, checked) in checkbox_q.iter() {
        if let Ok(mut border_color) = borders.get_mut(entity) {
            border_color.set_all(match *interaction {
                Interaction::Pressed => CHECKBOX_OUTLINE.lighter(PRESSED),
                Interaction::Hovered => CHECKBOX_OUTLINE.lighter(HOVERED),
                Interaction::None => CHECKBOX_OUTLINE,
            });
        }

        if let Ok(mut background_color) = marks.get_mut(entity) {
            *background_color = BackgroundColor(match checked {
                true => CHECKBOX_CHECK,
                false => Srgba::NONE.into(),
            });
        }
    }
}
