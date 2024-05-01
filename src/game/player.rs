use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    camera::{RenderLayer, YSorted},
    physics::Speed,
    playing, AppState,
};

use super::{
    resource_pool::{Fire, Health, ResourcePool},
    score_system::Score,
    InGameEntity, PLAYER_GROUP, POWERUP_GROUP, PROJECTILE_GROUP,
};

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMovementEvent>();

        app.add_systems(OnEnter(AppState::InGame), spawn_player);

        app.add_systems(FixedUpdate, handle_player_movement_events.run_if(playing()));
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub animation_indices: AnimationIndices,
    pub animation_timer: AnimationTimer,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub fire_breath_resource: ResourcePool<Fire>,
    pub hitpoints: ResourcePool<Health>,
    pub score: Score,
    pub damping: Damping,
    pub external_impulse: ExternalImpulse,
    pub marker: Player,
    pub rigid_body: RigidBody,
    pub speed: Speed,
    pub render_layers: RenderLayers,
    pub spritesheet: SpriteSheetBundle,
}

#[derive(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server
        .get_handle("textures/dragon.png")
        .unwrap_or_default();
    let texture_atlas_layout =
        TextureAtlasLayout::from_grid(Vec2::new(191., 161.), 12, 1, None, None);
    let texture_atlas_layout_handle = asset_server.add(texture_atlas_layout);

    let mut player_entity_commands = commands.spawn(PlayerBundle {
        animation_indices: AnimationIndices::new(0, 2),
        animation_timer: AnimationTimer::from_seconds(0.2),
        collider: Collider::cuboid(15., 40.),
        collision_groups: CollisionGroups::new(PLAYER_GROUP, PROJECTILE_GROUP | POWERUP_GROUP),
        fire_breath_resource: ResourcePool::<Fire>::new(100),
        hitpoints: ResourcePool::<Health>::new(100),
        score: Score::new(0, 1),
        marker: Player,
        render_layers: RenderLayers::layer(RenderLayer::Sky.into()),
        speed: Speed(10.),
        damping: Damping::default(),
        external_impulse: ExternalImpulse::default(),
        rigid_body: RigidBody::Dynamic,
        spritesheet: SpriteSheetBundle {
            atlas: TextureAtlas {
                layout: texture_atlas_layout_handle,
                index: 0,
            },
            texture,
            transform: Transform::from_translation(Vec2::ONE.extend(1.)),
            ..default()
        },
    });

    player_entity_commands.insert((InGameEntity, YSorted));
}

#[derive(Event)]
pub enum PlayerMovementEvent {
    Accelerate { target: Vec2 },
    Brake,
}

impl PlayerMovementEvent {
    pub fn accelerate(target: Vec2) -> Self {
        Self::Accelerate { target }
    }

    pub fn brake() -> Self {
        Self::Brake
    }
}

fn handle_player_movement_events(
    mut player_movement_event_reader: EventReader<PlayerMovementEvent>,
    mut query: Query<(&Transform, &mut ExternalImpulse, &mut Damping), With<Player>>,
) {
    let (transform, mut external_impulse, mut damping) = query.single_mut();

    for event in player_movement_event_reader.read() {
        match event {
            &PlayerMovementEvent::Accelerate { target } => {
                let player_position = transform.translation.truncate();
                let target_to_player_vector = target - player_position;

                if target_to_player_vector == Vec2::ZERO {
                    continue;
                }

                let target_distance_to_player = target.distance(player_position);
                let velocity_scalar = target_distance_to_player.min(300.) / 300.;
                let direction = transform.rotation.mul_vec3(Vec3::Y).truncate();
                let angle_with_cursor =
                    direction.angle_between(target_to_player_vector.normalize());
                let is_facing_forward = (-0.4..0.4).contains(&angle_with_cursor);
                let is_in_cruise_mode = (-0.2..0.2).contains(&angle_with_cursor);
                let angle = target_to_player_vector.angle_between(direction);

                external_impulse.impulse = direction * velocity_scalar * 125.;

                if is_facing_forward {
                    damping.angular_damping = 3.;
                    external_impulse.torque_impulse = 0.;
                    external_impulse.impulse *= 2.;
                    if is_in_cruise_mode {
                        damping.angular_damping = 5.;
                    }
                } else {
                    damping.angular_damping = 0.5;
                    external_impulse.torque_impulse = -angle * 2.;
                    damping.linear_damping = 1.5;
                }
            }
            PlayerMovementEvent::Brake => {
                *damping = Damping {
                    angular_damping: 25.,
                    linear_damping: 25.,
                };
            }
        }
    }
}
