use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use bevy::{prelude::*, sprite_render::TilemapChunkTileData, time::common_conditions::on_timer};
use wasmtime::{
    Config, Engine, Store,
    component::{Component, HasSelf, Linker},
};

use crate::{
    SPRITE_SIZE,
    api::{Pathfinding, TimelineAction, WasmRunner, host},
    goals::{Flag, Fox},
    map::{Map, MapPos},
};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum WasmState {
    Run,
    Error(String),
    #[default]
    Idle,
}

#[derive(Resource)]
pub struct WasmPathfinding {
    file: PathBuf,
    module: Pathfinding,
    store: Mutex<Store<WasmRunner>>,
}
impl WasmPathfinding {
    pub fn load(file: &PathBuf) -> Result<WasmPathfinding, wasmtime::Error> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        info!("Loading {}", file.display());

        let component = Component::from_file(&engine, file)?;
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

        let data = WasmRunner::default();
        let mut store = Store::new(component.engine(), data);
        host::add_to_linker::<_, HasSelf<_>>(&mut linker, |data: &mut WasmRunner| data)?;
        Ok(WasmPathfinding {
            file: file.clone(),
            module: Pathfinding::instantiate(&mut store, &component, &linker)?,
            store: Mutex::new(store),
        })
    }

    pub fn run(
        &mut self,
        tiles: Vec<Vec<bool>>,
        fox_pos: (u32, u32),
        flag_pos: (u32, u32),
    ) -> wasmtime::Result<()> {
        return self.module.guest().call_run(
            &mut (*self.store.lock().unwrap()),
            &tiles,
            fox_pos,
            flag_pos,
        );
    }
}

fn wasm_clean(
    mut commands: Commands,
    tiles_map: Single<(Entity, &TilemapChunkTileData)>,
    gizmos: Query<Entity, With<Gizmo>>,
) {
    let (entity, map) = *tiles_map;
    commands.entity(entity).insert(TilemapChunkTileData(
        map.iter()
            .map(|tile| {
                let mut tile = tile.unwrap_or_default();
                tile.color = Color::WHITE;
                Some(tile)
            })
            .collect(),
    ));

    gizmos.iter().for_each(|g| commands.entity(g).despawn());
}

fn wasm_run(
    mut wasm: ResMut<WasmPathfinding>,
    map: Res<Map>,
    fox_pos: Single<&MapPos, With<Fox>>,
    flag_pos: Single<&MapPos, With<Flag>>,
    mut mut_state: ResMut<NextState<WasmState>>,
) {
    println!("Fox position: {:?}", *fox_pos);
    println!("Flag position: {:?}", *flag_pos);

    if let Err(err) = wasm.run(
        map.to_pathfinding_map(),
        (**fox_pos).into(),
        (**flag_pos).into(),
    ) {
        error!("{}", err);
        mut_state.set(WasmState::Error(err.to_string()));
    }

    mut_state.set(WasmState::Idle);
}

const HALF_SIZE: Vec2 = vec2(SPRITE_SIZE as f32 / 2.0, SPRITE_SIZE as f32 / 2.0);

fn show_wasm_actions(
    mut commands: Commands,
    pathfinding: Res<WasmPathfinding>,
    mut map: ResMut<Map>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    pathfinding
        .store
        .lock()
        .unwrap()
        .data()
        .timeline
        .iter()
        .for_each(|entry| match *entry {
            TimelineAction::Line {
                start,
                end,
                color: (r, g, b),
            } => {
                let mut gizmo = GizmoAsset::default();

                let pos_start: MapPos = start.into();
                let pos_end: MapPos = end.into();

                let t_start: Transform = pos_start.into();
                let t_end: Transform = pos_end.into();

                gizmo.line_2d(
                    t_start.translation.xy() + HALF_SIZE,
                    t_end.translation.xy() + HALF_SIZE,
                    Color::srgb_u8(r, g, b),
                );

                commands.spawn(Gizmo {
                    handle: gizmo_assets.add(gizmo),
                    line_config: GizmoLineConfig {
                        width: 4.0,
                        ..default()
                    },
                    ..default()
                });
            }
            TimelineAction::Arrow {
                start,
                end,
                color: (r, g, b),
            } => {
                let mut gizmo = GizmoAsset::default();

                let pos_start: MapPos = start.into();
                let pos_end: MapPos = end.into();

                let t_start: Transform = pos_start.into();
                let t_end: Transform = pos_end.into();

                gizmo
                    .arrow_2d(
                        t_start.translation.xy() + HALF_SIZE,
                        t_end.translation.xy() + HALF_SIZE,
                        Color::srgb_u8(r, g, b),
                    )
                    .with_tip_length(SPRITE_SIZE as f32 / 2.0);

                commands.spawn(Gizmo {
                    handle: gizmo_assets.add(gizmo),
                    line_config: GizmoLineConfig {
                        width: 4.0,
                        ..default()
                    },
                    ..default()
                });
            }
            TimelineAction::Tile {
                pos: (pos_x, pos_y),
                color,
            } => {
                let color = Color::srgb_u8(color.0, color.1, color.2);
                let map_pos = MapPos { x: pos_x, y: pos_y };

                map.get_tile_mut(&map_pos).color = color.lighter(0.01);
            }
        });
}

#[derive(Resource, Default)]
pub struct WasmHotReloading(pub bool);

fn reload_if_modified(
    mut pathfinding: ResMut<WasmPathfinding>,
    mut mut_state: ResMut<NextState<WasmState>>,
) {
    let metadata = fs::metadata(pathfinding.file.clone()).unwrap();
    let modified = metadata.modified().unwrap();
    let age = SystemTime::now().duration_since(modified).unwrap();

    if age < Duration::from_secs(1) {
        info!("Reloading wasm...");
        *pathfinding = WasmPathfinding::load(&pathfinding.file).unwrap();
        mut_state.set(WasmState::Run);
    }
}

pub struct WasmRunnerPlugin;
impl Plugin for WasmRunnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<WasmState>()
            .add_systems(
                OnEnter(WasmState::Run),
                (wasm_clean, wasm_run, show_wasm_actions).chain(),
            )
            .init_resource::<WasmHotReloading>()
            .add_systems(
                Update,
                reload_if_modified
                    .run_if(
                        |hot_reloading: Res<WasmHotReloading>,
                         pathfinding: Option<Res<WasmPathfinding>>| {
                            hot_reloading.0 && pathfinding.is_some()
                        },
                    )
                    .run_if(on_timer(Duration::from_secs(1))),
            );
    }
}
