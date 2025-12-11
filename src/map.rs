use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};

use crate::{cursor::CursorState, ui::SPRITE_SIZE};

#[derive(Resource, Debug, Clone)]
pub struct Map(pub Vec<Vec<TileInfo>>);
impl Map {
    pub fn new(size: &MapSize) -> Self {
        Map(vec![
            vec![TileInfo::default(); size.0.x as usize];
            size.0.y as usize
        ])
    }

    pub fn new_from_old(old: &Map, old_size: &MapSize, new_size: &MapSize) -> Self {
        let (new_x, new_y) = new_size.0.into();
        let (prev_x, prev_y) = old_size.0.into();

        let mut new_map = Vec::with_capacity(new_y as usize);

        for y in 0..new_y {
            let mut row = Vec::with_capacity(new_x as usize);
            for x in 0..new_x {
                if y < prev_y && x < prev_x {
                    row.push(old.0[y as usize][x as usize]);
                } else {
                    row.push(TileInfo::default());
                }
            }
            new_map.push(row);
        }

        Map(new_map)
    }

    pub fn to_tilemap(&self) -> TilemapChunkTileData {
        TilemapChunkTileData(
            self.0
                .iter()
                .rev()
                // INVERT Y AXIS
                // X right Y up -> X right Y down
                // (Math coordinates -> array coordinates)
                .flatten()
                .map(|tile| {
                    Some(TileData {
                        tileset_index: tile.tile_type.to_index(),
                        color: tile.color,
                        visible: true,
                    })
                })
                .collect(),
        )
    }

    pub fn to_pathfinding_map(&self) -> Vec<Vec<bool>> {
        self.0
            .iter()
            .map(|row| {
                row.iter()
                    .map(|tile| tile.tile_type == TileType::Floor)
                    .collect()
            })
            .collect()
    }

    pub fn get_tile(&self, pos: &MapPos) -> &TileInfo {
        self.0
            .get(pos.y as usize)
            .unwrap()
            .get(pos.x as usize)
            .unwrap()
    }

    pub fn get_tile_mut(&mut self, pos: &MapPos) -> &mut TileInfo {
        self.0
            .get_mut(pos.y as usize)
            .unwrap()
            .get_mut(pos.x as usize)
            .unwrap()
    }
}

#[derive(Resource, Debug, Default, Copy, Clone)]
pub struct MapSize(pub UVec2);
impl MapSize {
    pub fn new(width: u32, height: u32) -> Self {
        MapSize(UVec2::new(width, height))
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TileInfo {
    pub tile_type: TileType,
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash, Default)]
pub enum TileType {
    #[default]
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
impl From<(u32, u32)> for MapPos {
    fn from(value: (u32, u32)) -> Self {
        MapPos {
            x: value.0,
            y: value.1,
        }
    }
}
impl From<MapPos> for (u32, u32) {
    fn from(value: MapPos) -> Self {
        (value.x, value.y)
    }
}
impl MapPos {
    pub fn clamp(self, size: &MapSize) -> MapPos {
        let (x, y) = size.0.into();
        MapPos {
            x: self.x.clamp(0, x - 1),
            y: self.y.clamp(0, y - 1),
        }
    }
    pub fn into_tile_index(self, size: &MapSize) -> usize {
        let (x, y) = size.0.into();

        let pos = self.clamp(size);
        ((y - pos.y - 1) * x + pos.x) as usize
    }
}

fn map_size_update(
    mut commands: Commands,
    old: Res<Map>,
    mut old_size: Local<MapSize>,
    new_size: Res<MapSize>,
) {
    commands.insert_resource(Map::new_from_old(&old, &old_size, &new_size));
    *old_size = *new_size;
}

fn map_render(
    mut commands: Commands,
    assets: Res<AssetServer>,
    size: Res<MapSize>,
    map: Res<Map>,
    maybe_map: Option<Single<Entity, With<TilemapChunk>>>,
) {
    if let Some(entity) = maybe_map {
        commands.entity(*entity).despawn();
    }

    // Need to always insert because TilemapChunkTileData is immutable
    // So respawning ~= inserting new TilemapChunkTileData
    commands
        .spawn((
            Transform::from_xyz(
                (SPRITE_SIZE * size.0.x) as f32 / 2.,
                (SPRITE_SIZE * size.0.y) as f32 / 2.,
                0.,
            ),
            TilemapChunk {
                tileset: assets.load("tiles.png"),
                chunk_size: size.0,
                tile_display_size: UVec2::splat(SPRITE_SIZE),
                ..default()
            },
            map.to_tilemap(),
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
        let map_size = MapSize::new(15, 10);

        app.insert_resource(map_size)
            .insert_resource(Map::new(&map_size))
            .add_systems(
                Update,
                (
                    map_size_update.run_if(|size: Res<MapSize>| size.is_changed()),
                    map_render.run_if(|map: Res<Map>| map.is_changed()),
                )
                    .chain(),
            )
            .add_systems(Update, update_tileset_image)
            .add_observer(map_pos_move);
    }
}
