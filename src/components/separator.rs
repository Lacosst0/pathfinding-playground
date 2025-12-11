use bevy::prelude::*;

pub fn separator() -> impl Bundle {
    (
        Node {
            height: px(2),
            width: percent(100.),
            align_self: AlignSelf::Stretch,
            ..default()
        },
        BackgroundColor(Color::srgb(0., 0., 0.)),
    )
}
