use bevy::{
    color::palettes::css::{FUCHSIA, RED, YELLOW},
    prelude::*,
};

use crate::{
    camera::MainCamera,
    game::{Player, GRID_SIZE, HALF_TILE_SIZE, TILE_SIZE},
    input::CursorWorldPositionChecker,
    playing,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (draw_grid, draw_camera_constraints, draw_mouse_direction).run_if(playing()),
        );
    }
}

fn draw_grid(mut gizmos: Gizmos) {
    gizmos.grid_2d(
        -HALF_TILE_SIZE,
        0.0,
        GRID_SIZE.as_uvec2(),
        TILE_SIZE,
        FUCHSIA,
    );
}

fn draw_camera_constraints(
    camera_query: Query<(&Camera, &Transform), (With<Camera2d>, With<MainCamera>)>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();

    if let Some(viewport_size) = camera.logical_viewport_size() {
        let level_dimensions = GRID_SIZE * TILE_SIZE;
        let viewport_size_remainder = viewport_size % TILE_SIZE;
        let camera_boundary_size = (level_dimensions
            - (viewport_size - viewport_size_remainder)
            - viewport_size_remainder)
            .clamp(Vec2::ZERO, Vec2::splat(f32::MAX));
        let camera_boundary = Rect::from_center_size(-HALF_TILE_SIZE, camera_boundary_size);

        gizmos.rect_2d(camera_boundary.center(), 0., camera_boundary.size(), RED);
        gizmos.circle_2d(camera_transform.translation.truncate(), 10., RED);
    }
}

fn draw_mouse_direction(
    mouse_input: ResMut<ButtonInput<MouseButton>>,
    cursor_world_position_checker: CursorWorldPositionChecker,
    query: Query<&Transform, With<Player>>,
    mut gizmos: Gizmos,
) {
    if mouse_input.pressed(MouseButton::Right) {
        if let Some(cursor_position) = cursor_world_position_checker.cursor_world_position() {
            let player_transform = query.single();
            let player_position = player_transform.translation.truncate();

            gizmos.line_2d(player_position, cursor_position, YELLOW);
        }
    }
}
