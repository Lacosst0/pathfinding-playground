use bevy::{prelude::*, sprite::Anchor, ui_widgets::observe};

use crate::{
    SPRITE_SIZE,
    cursor::CursorState,
    map::{Map, MapPos, MapSize, TileType},
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
    pos_q: Query<(Entity, &MapPos)>,
) {
    for (entity, &pos) in pos_q.iter() {
        let new_pos = pos.clamp(&size);

        commands
            .entity(entity)
            .insert_if(new_pos, || new_pos != pos);
    }
}

fn fix_goals_floor(mut map: ResMut<Map>, pos_q: Query<&MapPos>) {
    for pos in pos_q.iter() {
        if map.get_tile(pos).tile_type == TileType::Wall {
            map.get_tile_mut(pos).tile_type = TileType::Floor;
        }
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
            )
                .chain(),
        );
    }
}
