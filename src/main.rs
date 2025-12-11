//           ЭТОТ КОД ЗАЩИЩЕН ГИТПСОМ
//
//             @@                @@@
//            @@@@             @@@@@@
//           @@@@@@          @@@@@@@@
//           @@@@@@@        @@@@   @@
//           @@@@@@@@      @@@@    @@
//           @@     @@@@@@@@@@     @@
//             @@ @@@@@@@@@@@@@   @@@
//         @@@  @@@@@@@@@@@@@@@@@@@@@
//  @@@@@@@   @@@@@    @@@@@@@@@@@@@@
// @     @@@@@@@@@@@   @@@@@@@@@@@@@@
// @    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@
// @@ @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
//  @@ @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
//      @@@@@@@@@@@@  @@@@@@@@@@@@@@@
//     @            @@@@@@@@@@@@@@@@@
//       @@@@@@@@@@@@@@@@@@@@@@@@@@@
//            @@@           @@@@@@@
//                  @@@@@@@@@@@@@@
//               @@@@@@@@@@@@@@
//           @@@@@@@@@@@@@@@@@
//          @@@@@@@@@@@@@@@@
//         https://gitverse.ru

use bevy::{
    prelude::*,
    window::{PresentMode, Window},
    winit::{WINIT_WINDOWS, WinitWindows},
};
use image::imageops::FilterType;
use winit::window::Icon;

pub static SPRITE_SIZE: u32 = 16;

mod api;
mod components;
mod cursor;
mod goals;
mod map;
mod ui;
mod wasm;

fn startup(mut commands: Commands, size: Res<map::MapSize>) {
    let camera_id = commands
        .spawn((
            Camera2d,
            Projection::from(OrthographicProjection::default_2d()),
        ))
        .id();

    commands.entity(camera_id).insert(Transform::from_xyz(
        (size.0.x * SPRITE_SIZE) as f32 / 2.,
        (size.0.y * SPRITE_SIZE) as f32 / 2.,
        100.,
    ));
}

// I love bevy magic WIP
fn set_window_icon(
    _: Option<NonSend<WinitWindows>>, // put system on the same thread as WINIT_WINDOWS
) {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/icon.png")
            .expect("Failed to open icon path")
            .resize(256, 256, FilterType::Nearest)
            .into_rgba8();
        let (width, height) = image.dimensions();

        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    WINIT_WINDOWS.with_borrow_mut(|windows| {
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    });
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Pathfinding playground"),
                        present_mode: PresentMode::AutoNoVsync,
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(PickingPlugin {}),
        )
        .add_systems(Startup, startup)
        .add_systems(Update, set_window_icon)
        .add_plugins((
            cursor::CursorHandlerPlugin,
            components::ComponentsPlugin,
            ui::SettingsPlugin,
        ))
        .add_plugins((
            MeshPickingPlugin,
            map::MapHandlerPlugin,
            goals::GoalsHandlerPlugin,
        ))
        .add_plugins(wasm::WasmRunnerPlugin)
        .run();
}
