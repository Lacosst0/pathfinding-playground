use bevy::{
    prelude::*,
    ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue, ValueChange, observe},
};

use crate::components::{HOVERED, PRESSED};

const SLIDER_TRACK: Color = Color::srgb(0.05, 0.05, 0.05);
const SLIDER_THUMB: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn slider(min: f32, max: f32, default_value: f32) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Center,
            height: px(20),
            width: percent(100),
            ..default()
        },
        Slider::default(),
        SliderValue(default_value),
        SliderRange::new(min, max),
        Interaction::default(),
        observe(
            |value_change: On<ValueChange<f32>>, mut commands: Commands| {
                commands
                    .entity(value_change.source)
                    .insert(SliderValue(value_change.value));
            },
        ),
        children![
            // Slider background rail
            (
                Node {
                    height: px(8),
                    ..default()
                },
                BackgroundColor(SLIDER_TRACK), // Border color for the checkbox
                BorderRadius::all(px(3)),
            ),
            // Invisible track to allow absolute placement of thumb entity. This is narrower than
            // the actual slider, which allows us to position the thumb entity using simple
            // percentages, without having to measure the actual width of the slider thumb.
            (
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    bottom: px(0),
                    // Track is short by thumb width to accommodate the thumb.
                    right: px(20),
                    ..default()
                },
                children![(
                    SliderThumb,
                    Node {
                        display: Display::Flex,
                        width: px(20),
                        height: px(20),
                        position_type: PositionType::Absolute,
                        left: percent(0), // This will be updated by the slider's value
                        ..default()
                    },
                    BorderRadius::MAX,
                    BackgroundColor(SLIDER_THUMB),
                )],
            ),
        ],
    )
}

pub(super) fn update_slider_style(
    slider_q: Query<
        (Entity, &SliderValue, &SliderRange, &Interaction),
        (
            Or<(Changed<Interaction>, Changed<SliderValue>)>,
            With<Slider>,
        ),
    >,
    children: Query<&Children>,
    mut thumbs: Query<(&mut Node, &mut BackgroundColor), With<SliderThumb>>,
) {
    for (slider_ent, value, range, interaction) in slider_q.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, mut color)) = thumbs.get_mut(child) {
                thumb_node.left = percent(range.thumb_position(value.0) * 100.0);
                *color = (match *interaction {
                    Interaction::Pressed => SLIDER_THUMB.lighter(PRESSED),
                    Interaction::Hovered => SLIDER_THUMB.lighter(HOVERED),
                    Interaction::None => SLIDER_THUMB,
                })
                .into();
            }
        }
    }
}
