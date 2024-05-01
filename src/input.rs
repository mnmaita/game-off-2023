use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

use crate::{
    camera::MainCamera, game::Player, game::PlayerMovementEvent, game::SpawnFireBreathEvent,
    playing, AppState,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                clear_input.run_if(state_changed::<AppState>),
                mouse_input.run_if(playing()),
            )
                .chain(),
        );
    }
}

#[derive(Resource)]
struct CursorWorldPosition(Option<Vec2>);

#[derive(SystemParam)]
pub struct CursorWorldPositionChecker<'w, 's> {
    window_query: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera_query: Query<
        'w,
        's,
        (&'static Camera, &'static GlobalTransform),
        (With<Camera2d>, With<MainCamera>),
    >,
}

impl CursorWorldPositionChecker<'_, '_> {
    pub fn cursor_world_position(&self) -> Option<Vec2> {
        let window = self.window_query.single();

        window.cursor_position().and_then(|cursor_position| {
            let (camera, camera_transform) = self.camera_query.single();
            camera.viewport_to_world_2d(camera_transform, cursor_position)
        })
    }
}

#[derive(Component)]
struct FireBreathSfx;

fn mouse_input(
    mut commands: Commands,
    mut spawn_fire_breath_event_writer: EventWriter<SpawnFireBreathEvent>,
    mut player_movement_event_writer: EventWriter<PlayerMovementEvent>,
    cursor_world_position_checker: CursorWorldPositionChecker,
    query: Query<&Transform, With<Player>>,
    sfx_query: Query<Entity, With<FireBreathSfx>>,
    asset_server: Res<AssetServer>,
    mouse_input: ResMut<ButtonInput<MouseButton>>,
) {
    if let Some(cursor_position) = cursor_world_position_checker.cursor_world_position() {
        player_movement_event_writer.send(PlayerMovementEvent::accelerate(cursor_position));
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        commands.spawn((
            AudioBundle {
                source: asset_server
                    .get_handle("sfx/breathstart.ogg")
                    .unwrap_or_default(),
                settings: PlaybackSettings::DESPAWN,
            },
            FireBreathSfx,
        ));
        commands.spawn((
            AudioBundle {
                source: asset_server
                    .get_handle("sfx/breathloop.ogg")
                    .unwrap_or_default(),
                settings: PlaybackSettings::LOOP,
            },
            FireBreathSfx,
        ));
    } else if mouse_input.just_released(MouseButton::Left) {
        commands.spawn((
            AudioBundle {
                source: asset_server
                    .get_handle("sfx/breathend.ogg")
                    .unwrap_or_default(),
                settings: PlaybackSettings::DESPAWN,
            },
            FireBreathSfx,
        ));
        for entity in &sfx_query {
            commands.entity(entity).despawn_recursive();
        }
    }

    if mouse_input.pressed(MouseButton::Left) {
        let player_transform = query.single();
        let player_direction = player_transform.rotation.mul_vec3(Vec3::Y).truncate();
        // TODO: replace constant with sprite dimensions
        let fire_position = player_transform.translation.truncate() + player_direction * 90.;

        spawn_fire_breath_event_writer.send(SpawnFireBreathEvent::new(1, fire_position));
    }
}

fn clear_input(
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut mouse_input: ResMut<ButtonInput<MouseButton>>,
) {
    keyboard_input.reset_all();
    mouse_input.reset_all()
}
