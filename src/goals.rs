use bevy::{prelude::*, sprite::Anchor, sprite_render::TilemapChunkTileData, ui_widgets::observe};

use crate::{
    cursor::CursorState,
    map::{MapPos, MapSize, TileType},
    ui::SPRITE_SIZE,
};

// Tag components for goals
#[derive(Component)]
pub struct Fox;
#[derive(Component)]
pub struct Flag;

fn map_load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    size: Res<MapSize>,
    mut picking_settings: ResMut<SpritePickingSettings>,
) {
    picking_settings.picking_mode = SpritePickingMode::BoundingBox;

    commands.spawn((
        Sprite {
            image: asset_server.load("fox.png"),
            custom_size: Some(Vec2::splat(SPRITE_SIZE as f32 * 1.1)), // make bigger to be seen better
            ..default()
        },
        MapPos { x: 0, y: 0 },
        Fox,
        Pickable::default(),
        Anchor::BOTTOM_LEFT,
        observe(
            |event: On<Pointer<Press>>, mut state: ResMut<NextState<CursorState>>| {
                state.set(CursorState::Dragging(event.entity));
            },
        ),
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("flag.png"),
            custom_size: Some(Vec2::splat(SPRITE_SIZE as f32 * 1.1)), // make bigger to be seen better
            ..default()
        },
        MapPos {
            x: size.0.x - 1,
            y: size.0.y - 1,
        },
        Flag,
        Pickable::default(),
        Anchor::BOTTOM_LEFT,
        observe(
            |event: On<Pointer<Press>>, mut state: ResMut<NextState<CursorState>>| {
                state.set(CursorState::Dragging(event.entity));
            },
        ),
    ));
}

fn fix_goals_positions(
    mut commands: Commands,
    size: Res<MapSize>,
    fox_pos: Single<(Entity, &MapPos), With<Fox>>,
    flag_pos: Single<(Entity, &MapPos), With<Flag>>,
) {
    for (entity, &pos) in [*fox_pos, *flag_pos] {
        let (x, y) = size.0.into();
        let new_pos = pos.clamp(x, y);

        commands
            .entity(entity)
            .insert_if(new_pos, || new_pos != pos);
    }
}

fn fix_goals_collision(
    mut commands: Commands,
    fox: Single<(Entity, &MapPos), With<Fox>>,
    size: Res<MapSize>,
) {
    let (x, y) = size.0.into();
    let (entity, &pos) = *fox;
    let mut new_pos = pos.clamp(x, y);

    if pos.x == x - 1 {
        new_pos.x -= 1;
    } else {
        new_pos.x += 1;
    }
    if pos.y == y - 1 {
        new_pos.y -= 1;
    } else {
        new_pos.y += 1;
    }

    commands.entity(entity).insert(new_pos);
}

fn fix_goals_floor(
    size: Res<MapSize>,
    mut tiles_map: Single<&mut TilemapChunkTileData>,
    fox_pos: Single<&MapPos, With<Fox>>,
    flag_pos: Single<&MapPos, With<Flag>>,
) {
    for pos in [*fox_pos, *flag_pos] {
        let tile_index = pos.into_tile_index(&size);

        let mut tile = tiles_map[tile_index].unwrap();
        tile.tileset_index = TileType::Floor.to_index();

        tiles_map[tile_index] = Some(tile);
    }
}

pub struct GoalsHandlerPlugin;
impl Plugin for GoalsHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, map_load).add_systems(
            Update,
            (
                fix_goals_positions.run_if(|size: Res<MapSize>| size.is_changed()),
                fix_goals_floor.run_if(in_state(CursorState::Idle)),
                fix_goals_collision.run_if(
                    |fox_pos: Single<&MapPos, With<Fox>>, flag_pos: Single<&MapPos, With<Flag>>| {
                        *fox_pos == *flag_pos
                    },
                ),
            )
                .chain(),
        );
    }
}
