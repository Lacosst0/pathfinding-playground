use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::map::{Map, MapPos, MapSize, TileType};

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
    map_pos_q: Query<(Entity, &MapPos)>,
    size: Res<MapSize>,
) {
    if let CursorState::Dragging(event_entity) = *state.get() {
        let event_map_pos: MapPos = Transform::from_translation(cursor_pos.0.extend(1.)).into();

        if !map_pos_q
            .iter()
            .any(|(entity, map_pos)| event_entity != entity && *map_pos == event_map_pos)
        {
            commands
                .entity(event_entity)
                .insert(event_map_pos.clamp(&size));
        }
    }
}

fn cursor_placing(
    state: Res<State<CursorState>>,
    cursor_pos: Res<CursorPos>,
    mut map: ResMut<Map>,
    size: Res<MapSize>,
) {
    if let CursorState::Placing(tile_type) = *state.get() {
        let map_pos: MapPos = Transform::from_translation(cursor_pos.0.extend(0.)).into();
        let pos = &map_pos.clamp(&size);

        if map.get_tile(pos).tile_type != tile_type {
            map.get_tile_mut(pos).tile_type = tile_type;
        }
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
            .add_systems(Update, middle_zoom)
            .add_observer(middle_move)
            .add_observer(
                |_: On<Pointer<Release>>, mut state: ResMut<NextState<CursorState>>| {
                    state.set(CursorState::Idle);
                },
            );
    }
}
