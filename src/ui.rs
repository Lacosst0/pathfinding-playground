use core::time::Duration;
use std::env::current_dir;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    math::Vec2,
    prelude::*,
    sprite_render::{TileData, TilemapChunkTileData},
    time::common_conditions::on_timer,
    ui_widgets::{
        Activate, Button, Slider, SliderRange, SliderThumb, SliderValue, UiWidgetsPlugins,
        ValueChange, observe,
    },
};
use rfd::FileDialog;

use crate::{
    goals::Fox,
    map::MapSize,
    wasm::{WasmPathfinding, WasmState},
};

pub static SPRITE_SIZE: u32 = 16;

const BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const SLIDER_TRACK: Color = Color::srgb(0.05, 0.05, 0.05);
const SLIDER_THUMB: Color = Color::srgb(0.35, 0.75, 0.35);

const HOVERED: f32 = 0.01;
const PRESSED: f32 = 0.1;

fn text(text: &str, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
    )
}

fn button(text: impl Bundle) -> impl Bundle {
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

fn update_button_style(
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

fn slider(min: f32, max: f32, default_value: f32) -> impl Bundle {
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

fn update_slider_style(
    slider_q: Query<
        (Entity, &SliderValue, &SliderRange, &Interaction),
        (
            Or<(Changed<Interaction>, Changed<SliderValue>)>,
            With<Slider>,
        ),
    >,
    children: Query<&Children>,
    mut thumbs: Query<(&mut Node, &mut BackgroundColor, Has<SliderThumb>)>,
) {
    for (slider_ent, value, range, interaction) in slider_q.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, mut color, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
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

fn separator() -> impl Bundle {
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
fn ui_startup(mut commands: Commands) {
    commands.spawn((
        Node {
            display: Display::Flex,
            width: percent(20),
            height: vh(100),
            border: Val::right(px(2)),
            padding: Val::all(px(8)),
            row_gap: px(8),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        BorderColor {
            right: Color::BLACK,
            ..default()
        },
        children![
            (text("FPS: ?", 32.), FPSText),
            separator(),
            (
                button(text("Center camera", 24.)),
                observe(
                    |_: On<Activate>,
                     // no double transform use
                     mut camera: Single<&mut Transform, With<Camera>>,
                     fox: Single<&Transform, (With<Fox>, Without<Camera>)>| {
                        camera.translation =
                            fox.translation + Vec2::splat(SPRITE_SIZE as f32 / 2.).extend(0.);
                    }
                )
            ),
            (
                button(text("Reset walls", 24.)),
                observe(
                    |_: On<Activate>,
                     mut commands: Commands,
                     chunk_entity: Single<Entity, With<TilemapChunkTileData>>,
                     size: Res<MapSize>| {
                        let tile_map: Vec<Option<TileData>> = (0..size.0.element_product())
                            .map(|_| Some(TileData::default()))
                            .collect();

                        commands
                            .entity(*chunk_entity)
                            .insert(TilemapChunkTileData(tile_map));
                    }
                )
            ),
            separator(),
            (text("...", 24.), SelectAlgorithmText),
            (
                button(text("Select algorithm", 24.)),
                observe(
                    |_: On<Activate>,
                     mut commands: Commands,
                     mut text: Single<&mut Text, With<SelectAlgorithmText>>,

                     mut mut_state: ResMut<NextState<WasmState>>| {
                        if let Some(file) = FileDialog::new()
                            .add_filter("WebAssembly", &["wasm"])
                            .set_directory(current_dir().unwrap().join("algorithms"))
                            .pick_file()
                        {
                            match WasmPathfinding::load(&file) {
                                Ok(wasm) => {
                                    commands.insert_resource(wasm);
                                    text.0 =
                                        format!("{}", file.file_name().unwrap().to_string_lossy());
                                    mut_state.set(WasmState::Run);
                                }
                                Err(err) => {
                                    error!("{}", err);
                                    text.0 = "Error loading wasm".to_owned();
                                }
                            }
                        }
                    }
                )
            ),
            (
                button(text("Clear map", 24.)),
                observe(
                    |_: On<Activate>,
                     mut commands: Commands,
                     tiles_map: Single<(Entity, &TilemapChunkTileData)>,
                     gizmos: Query<Entity, With<Gizmo>>| {
                        commands.entity(tiles_map.0).insert(TilemapChunkTileData(
                            tiles_map
                                .1
                                .iter()
                                .map(|tile| {
                                    let mut tile = tile.unwrap_or_default();
                                    tile.color = Color::WHITE;
                                    Some(tile)
                                })
                                .collect(),
                        ));

                        gizmos.iter().for_each(|g| commands.entity(g).despawn());
                    }
                )
            ),
            separator(),
            (text("Map size: 0x0", 32.), MapSizeText),
            (
                Node {
                    display: Display::Flex,
                    width: percent(100),
                    ..default()
                },
                children![
                    text("X:", 24.),
                    (
                        slider(2., 128., 10.),
                        observe(
                            |value_change: On<ValueChange<f32>>, mut map_size: ResMut<MapSize>| {
                                map_size.0.x = value_change.value as u32;
                            },
                        ),
                    )
                ]
            ),
            (
                Node {
                    display: Display::Flex,
                    width: percent(100),
                    ..default()
                },
                children![
                    text("Y:", 24.),
                    (
                        slider(2., 128., 10.),
                        observe(
                            |value_change: On<ValueChange<f32>>, mut map_size: ResMut<MapSize>| {
                                map_size.0.y = value_change.value as u32;
                            },
                        ),
                    )
                ]
            ),
        ],
    ));
}

#[derive(Component)]
struct FPSText;

#[derive(Component)]
struct SelectAlgorithmText;

#[derive(Component)]
struct MapSizeText;

fn fps_text_update(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text: Single<&mut Text, With<FPSText>>,
) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        && let Some(value) = fps.smoothed()
    {
        fps_text.0 = format!("FPS: {value:.0}");
    }
}

fn map_size_text_update(
    map_size: Res<MapSize>,
    mut map_size_text: Single<&mut Text, With<MapSizeText>>,
) {
    if map_size.is_changed() {
        map_size_text.0 = format!("Map Size: {}x{}", map_size.0.x, map_size.0.y);
    }
}

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiWidgetsPlugins)
            // UI
            .add_systems(Startup, ui_startup)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(
                Update,
                fps_text_update.run_if(on_timer(Duration::from_secs_f32(0.5))),
            )
            .add_systems(Update, map_size_text_update)
            .add_systems(Update, update_button_style)
            .add_systems(Update, update_slider_style);
    }
}
