use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;

use crate::{
    camera::{RenderLayer, YSorted},
    playing, AppState,
};

use super::{
    power_up::{PowerUpEvent, PowerUpEventType},
    resource_pool::{Fire, Health, ResourcePool},
    score_system::{ScoreEvent, ScoreEventType},
    Enemy, Player, PLAYER_GROUP, PROJECTILE_GROUP, TILE_SIZE,
};

pub(super) struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnProjectileEvent>();

        app.add_systems(
            FixedUpdate,
            (
                projectile_collision_with_player,
                spawn_projectiles,
                despawn_projectiles,
                despawn_dead_entities,
                compute_damage_from_intersections,
            )
                .run_if(playing()),
        );
    }
}

#[derive(Event)]
pub struct SpawnProjectileEvent {
    damage: i16,
    direction: Vec2,
    emitter: Entity,
    position: Vec2,
    speed: f32,
}

impl SpawnProjectileEvent {
    pub fn new(damage: i16, direction: Vec2, emitter: Entity, position: Vec2, speed: f32) -> Self {
        Self {
            damage,
            direction,
            emitter,
            position,
            speed,
        }
    }
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub ccd: Ccd,
    pub damage: ImpactDamage,
    pub emitter: Emitter,
    pub marker: Projectile,
    pub render_layers: RenderLayers,
    pub rigid_body: RigidBody,
    pub sprite: SpriteBundle,
    pub velocity: Velocity,
}

#[derive(Component)]
pub struct Emitter(Entity);

#[derive(Component)]
pub struct Range(pub f32);

/// Represents the damage this entity causes to others when colliding.
#[derive(Component)]
pub struct ImpactDamage(pub i16);

/// Represents an Entity's damage attributes.
#[derive(Component)]
pub struct AttackDamage(pub i16);

#[derive(Component, Deref, DerefMut)]
pub struct AttackTimer(Timer);

impl AttackTimer {
    pub fn new(seconds: f32) -> Self {
        Self(Timer::from_seconds(seconds, TimerMode::Repeating))
    }
}

#[derive(Component)]
pub struct Projectile;

fn spawn_projectiles(
    mut commands: Commands,
    mut spawn_projectile_event_reader: EventReader<SpawnProjectileEvent>,
) {
    for &SpawnProjectileEvent {
        damage,
        direction,
        emitter,
        position,
        speed,
    } in spawn_projectile_event_reader.read()
    {
        let size = Vec2::new(TILE_SIZE.x, 4.);
        let angle = if direction != Vec2::ZERO {
            let mut angle = (direction).angle_between(Vec2::X);
            if !angle.is_finite() {
                angle = 0.;
            }
            angle
        } else {
            0.
        };

        commands.spawn((
            ProjectileBundle {
                ccd: Ccd::enabled(),
                collider: Collider::cuboid(size.x / 2., size.y / 2.),
                collision_groups: CollisionGroups::new(
                    PROJECTILE_GROUP,
                    PLAYER_GROUP | PROJECTILE_GROUP,
                ),
                damage: ImpactDamage(damage),
                emitter: Emitter(emitter),
                marker: Projectile,
                render_layers: RenderLayers::layer(RenderLayer::Sky.into()),
                rigid_body: RigidBody::Dynamic,
                sprite: SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(size),
                        ..default()
                    },
                    transform: Transform::from_translation(position.extend(1.0))
                        .with_rotation(Quat::from_rotation_z(-angle)),
                    ..default()
                },
                velocity: Velocity {
                    linvel: direction * speed,
                    angvel: 0.,
                },
            },
            Damping {
                linear_damping: 1.0,
                angular_damping: 10.0,
            },
            StateScoped(AppState::GameOver),
            YSorted,
        ));
    }
}

fn projectile_collision_with_player(
    mut commands: Commands,
    mut score_event_writer: EventWriter<ScoreEvent>,
    mut player_query: Query<(Entity, &mut ResourcePool<Health>), With<Player>>,
    projectile_query: Query<(Entity, &ImpactDamage), With<Projectile>>,
    rapier_context: Res<RapierContext>,
) {
    let (player_entity, mut player_hitpoints) = player_query.single_mut();

    for (projectile_entity, projectile_damage) in &projectile_query {
        if let Some(contact_pair) = rapier_context.contact_pair(player_entity, projectile_entity) {
            if contact_pair.has_any_active_contact() {
                player_hitpoints.subtract(projectile_damage.0);
                score_event_writer.send(ScoreEvent::new(0, ScoreEventType::ResetMultiplier));

                // TODO: Add "death" component or event and use it here so a different system handles despawns.
                commands.entity(projectile_entity).despawn_recursive();
            }
        }
    }
}

fn compute_damage_from_intersections(
    mut enemy_query: Query<&mut ResourcePool<Health>, With<Enemy>>,
    fire_query: Query<(Entity, &ImpactDamage), With<Fire>>,
    rapier_context: Res<RapierContext>,
) {
    for (entity, damage) in &fire_query {
        for (entity1, entity2, intersecting) in rapier_context.intersection_pairs_with(entity) {
            let other_entity = if entity1 == entity { entity2 } else { entity1 };

            if intersecting {
                if let Ok(mut enemy_hitpoints) = enemy_query.get_mut(other_entity) {
                    enemy_hitpoints.subtract(damage.0);
                }
            }
        }
    }
}

fn despawn_dead_entities(
    mut commands: Commands,
    mut score_event_writer: EventWriter<ScoreEvent>,
    mut powerup_event_writer: EventWriter<PowerUpEvent>,
    query: Query<
        (Entity, &ResourcePool<Health>, &Transform),
        (Without<Player>, Changed<ResourcePool<Health>>),
    >,
) {
    for (entity, health, transform) in &query {
        if health.current() == 0 {
            commands.entity(entity).despawn_recursive();
            score_event_writer.send(ScoreEvent::new(10, ScoreEventType::AddPoints));
            powerup_event_writer.send(PowerUpEvent::new(
                *transform,
                PowerUpEventType::HealingScale,
            ));
        }
    }
}

fn despawn_projectiles(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Velocity), (With<Projectile>, Changed<Velocity>)>,
) {
    for (entity, velocity) in &projectile_query {
        if velocity.linvel.length() < 60. {
            // TODO: Decouple this with a Despawn component
            commands.entity(entity).despawn_recursive();
        }
    }
}
