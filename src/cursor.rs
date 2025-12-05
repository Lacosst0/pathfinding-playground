use bevy::{input::mouse::MouseWheel, prelude::*, sprite_render::TilemapChunkTileData};

use crate::map::{MapPos, MapSize, TileType};

#[derive(Resource, Default)]
pub struct CursorPos(pub Vec2);

// Not using `Drag` because:
// 1) Need "On ENTITY Press; until GLOBAL Release"
// 2) `Drag` is fired only on Move
//
// Goals => Dragging(goal_entity)
// Tiles => Placing(mouse_button => tile_type)
// `Release` => Idle
#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CursorState {
    Dragging(Entity),
    Placing(TileType),
    #[default]
    Idle,
}

fn set_cursor_pos(
    window: Single<&Window>,
    camera_q: Single<(&Camera, &GlobalTransform), With<Camera>>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    let (camera, camera_transform) = *camera_q;

    if let Some(vec_2) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        *cursor_pos = CursorPos(vec_2);
    }
}

fn cursor_dragging(
    mut commands: Commands,
    state: Res<State<CursorState>>,
    cursor_pos: Res<CursorPos>,
    size: Res<MapSize>,
) {
    if let CursorState::Dragging(entity) = *state.get() {
        let map_pos: MapPos = Transform::from_translation(cursor_pos.0.extend(1.)).into();

        commands
            .entity(entity)
            .insert(map_pos.clamp(size.0.x, size.0.y));
    }
}

fn cursor_placing(
    state: Res<State<CursorState>>,
    cursor_pos: Res<CursorPos>,
    size: Res<MapSize>,
    mut tiles_map: Single<&mut TilemapChunkTileData>,
) {
    if let CursorState::Placing(tile_type) = *state.get() {
        let map_pos: MapPos = Transform::from_translation(cursor_pos.0.extend(0.)).into();
        let tile_index = map_pos.into_tile_index(&size);

        let mut tile = tiles_map[tile_index].unwrap();
        tile.tileset_index = tile_type.to_index();

        tiles_map[tile_index] = Some(tile);
    }
}

fn middle_zoom(
    mut mouse_ev: MessageReader<MouseWheel>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    for ev in mouse_ev.read() {
        camera.scale *= Vec2::splat(1.0 - ev.y * 0.1).extend(1.0);
    }
}

fn middle_move(event: On<Pointer<Drag>>, mut camera: Single<&mut Transform, With<Camera>>) {
    if event.button == PointerButton::Middle {
        let scale = camera.scale;
        camera.translation += (event.delta * vec2(-0.5, 0.5)).extend(0.0) * scale;
    }
}

pub struct CursorHandlerPlugin;
impl Plugin for CursorHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<CursorState>()
            .insert_resource(CursorPos::default())
            .add_systems(Update, set_cursor_pos)
            .add_systems(Update, cursor_dragging)
            .add_systems(Update, cursor_placing)
            // middle actions
            .add_systems(Update, middle_zoom)
            .add_observer(middle_move)
            .add_observer(
                |_: On<Pointer<Release>>, mut state: ResMut<NextState<CursorState>>| {
                    state.set(CursorState::Idle);
                },
            );
    }
}
