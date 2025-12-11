use core::time::Duration;
use std::env::current_dir;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    math::Vec2,
    prelude::*,
    time::common_conditions::on_timer,
    ui_widgets::{Activate, UiWidgetsPlugins, ValueChange, observe},
};
use rfd::FileDialog;

use crate::{
    SPRITE_SIZE,
    components::*,
    goals::Fox,
    map::{Map, MapSize},
    wasm::{WasmPathfinding, WasmState},
};

fn ui_startup(mut commands: Commands, map_size: Res<MapSize>) {
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
                    |_: On<Activate>, mut commands: Commands, size: Res<MapSize>| {
                        commands.insert_resource(Map::new(&size));
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
                        slider(2., 128., map_size.0.x as f32),
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
                        slider(2., 128., map_size.0.y as f32),
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
struct SelectAlgorithmText;

#[derive(Component)]
struct FPSText;

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

#[derive(Component)]
struct MapSizeText;

fn map_size_text_update(size: Res<MapSize>, mut size_text: Single<&mut Text, With<MapSizeText>>) {
    size_text.0 = format!("Map Size: {}x{}", size.0.x, size.0.y);
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
            .add_systems(
                Update,
                map_size_text_update.run_if(|size: Res<MapSize>| size.is_changed()),
            );
    }
}
