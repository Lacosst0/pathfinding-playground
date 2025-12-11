use bevy::prelude::*;

pub fn text(text: &str, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
    )
}
