use std::cmp::min;

use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

use crate::{cursor::CursorState, ui::SPRITE_SIZE};

#[derive(Resource, Debug, Default, Copy, Clone)]
pub struct MapSize(pub UVec2);
impl MapSize {
    pub fn new(width: u32, height: u32) -> Self {
        MapSize(UVec2::new(width, height))
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum TileType {
    Floor,
    Wall,
}
impl TileType {
    pub fn to_index(self) -> u16 {
        match self {
            TileType::Floor => 0,
            TileType::Wall => 1,
        }
    }
}


#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct MapPos {
    pub x: u32,
    pub y: u32,
}
impl From<MapPos> for Vec2 {
    fn from(val: MapPos) -> Vec2 {
        Vec2::new((val.x * SPRITE_SIZE) as f32, (val.y * SPRITE_SIZE) as f32)
    }
}
impl From<MapPos> for Transform {
    fn from(val: MapPos) -> Self {
        Transform::from_xyz(
            (val.x * SPRITE_SIZE) as f32,
            (val.y * SPRITE_SIZE) as f32,
            1.,
        )
    }
}
impl From<Transform> for MapPos {
    fn from(value: Transform) -> Self {
        MapPos {
            x: value.translation.x as u32 / SPRITE_SIZE,
            y: (value.translation.y as u32 / SPRITE_SIZE),
        }
    }
}
impl From<MapPos> for UVec2 {
    fn from(val: MapPos) -> Self {
        UVec2 { x: val.x, y: val.y }
    }
}
impl From<UVec2> for MapPos {
    fn from(val: UVec2) -> Self {
        MapPos { x: val.x, y: val.y }
    }
}
impl MapPos {
    pub fn clamp(self, x: u32, y: u32) -> MapPos {
        MapPos {
            x: self.x.clamp(0, x - 1),
            y: self.y.clamp(0, y - 1),
        }
    }
    pub fn into_tile_index(self, size: &MapSize) -> usize {
        let (x, y) = size.0.into();

        let pos = self.clamp(x, y);
        ((y - pos.y - 1) * x + pos.x) as usize
    }
}

fn map_load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    size: Res<MapSize>,
    mut prev_size: Local<MapSize>,
    maybe_map: Option<Single<Entity, With<TilemapChunk>>>,
    chunk_data: Query<&TilemapChunkTileData>,
) {
    let mut tile_map: Vec<Option<TileData>> = (0..size.0.element_product())
        .map(|_| Some(TileData::default()))
        .collect();

    if let Some(map) = maybe_map {
        let data = chunk_data.get(*map).unwrap().0.clone();

        let (new_x, new_y) = size.0.into();
        let (prev_x, prev_y) = prev_size.0.into();

        for x in 0..min(prev_x, new_x) {
            for y in 0..min(prev_y, new_y) {
                let new_pos = ((new_y - y - 1) * new_x + x) as usize;
                let prev_pos = ((prev_y - y - 1) * prev_x + x) as usize;

                tile_map[new_pos] = data[prev_pos];
            }
        }

        commands.entity(*map).despawn();
    }

    commands
        .spawn((
            TilemapChunk {
                tileset: assets.load("tiles.png"),
                // the dimensions of the chunk (in tiles)
                chunk_size: size.0,
                // the size to render each tile (in pixels)
                tile_display_size: UVec2::splat(SPRITE_SIZE),
                ..default()
            },
            TilemapChunkTileData(tile_map),
            Transform::from_xyz(
                (SPRITE_SIZE * size.0.x) as f32 / 2.,
                (SPRITE_SIZE * size.0.y) as f32 / 2.,
                0.,
            ),
            Pickable::default(),
        ))
        .observe(
            |event: On<Pointer<Press>>, mut state: ResMut<NextState<CursorState>>| {
                state.set(match event.button {
                    PointerButton::Primary => CursorState::Placing(TileType::Wall),
                    PointerButton::Secondary => CursorState::Placing(TileType::Floor),
                    PointerButton::Middle => CursorState::Idle,
                });
            },
        );

    *prev_size = *size;
}

fn update_tileset_image(
    chunk_q: Single<&TilemapChunk>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        let chunk = *chunk_q;
        if event.is_loaded_with_dependencies(chunk.tileset.id()) {
            let image = images.get_mut(&chunk.tileset).unwrap();
            image.reinterpret_stacked_2d_as_array(2);
        }
    }
}

fn map_pos_move(event: On<Insert, MapPos>, mut commands: Commands, map_pos_q: Query<&MapPos>) {
    let global_pos: Transform = (*map_pos_q.get(event.entity).unwrap()).into();

    commands.entity(event.entity).insert(global_pos);
}

pub struct MapHandlerPlugin;
impl Plugin for MapHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapSize::new(15, 10))
            .add_systems(
                Update,
                map_load.run_if(|size: Res<MapSize>| size.is_changed()),
            )
            .add_systems(Update, update_tileset_image)
            .add_observer(map_pos_move);
    }
}
