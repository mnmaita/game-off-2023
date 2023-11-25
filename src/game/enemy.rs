use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;
use rand::seq::IteratorRandom;

use crate::{
    camera::{YSorted, GROUND_LAYER},
    physics::Speed,
    playing,
};

use super::{
    combat::{AttackDamage, AttackTimer, Range, SpawnProjectileEvent},
    resource_pool::{Health, ResourcePool},
    BorderTile, InGameEntity, Player, BUILDING_GROUP, ENEMY_GROUP, FIRE_BREATH_GROUP,
    HALF_TILE_SIZE, TILE_SIZE,
};

pub(super) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer::new(3.));

        app.add_systems(
            FixedUpdate,
            (spawn_enemies, handle_enemy_behavior, handle_enemy_attacks).run_if(playing()),
        );
    }
}

#[derive(Bundle)]
pub struct EnemyBundle {
    pub attack_damage: AttackDamage,
    pub attack_timer: AttackTimer,
    pub behavior: Behavior,
    pub hitpoints: ResourcePool<Health>,
    pub marker: Enemy,
    pub range: Range,
    pub speed: Speed,
    pub sprite: SpriteSheetBundle,
    pub collider: Collider,
    pub render_layers: RenderLayers,
    pub rigid_body: RigidBody,
    pub collision_groups: CollisionGroups,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Resource, Deref, DerefMut)]
struct EnemySpawnTimer(Timer);

impl EnemySpawnTimer {
    pub fn new(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Repeating))
    }
}

#[derive(Component)]
pub enum Behavior {
    FollowPlayer { distance: f32 },
}

fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    tile_query: Query<&Transform, With<BorderTile>>,
) {
     let texture = asset_server
        .get_handle("textures/enemy_archer.png")
        .unwrap_or_default();
    let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(50., 75.), 32, 8, None, None);
    let texture_atlas_handle = asset_server.add(texture_atlas);

    if enemy_spawn_timer.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        if let Some(tile_transform) = tile_query.iter().choose(&mut rng) {
            let translation = tile_transform.translation.truncate().extend(1.);
            let mut enemy_entity_commands = commands.spawn(EnemyBundle {
                attack_damage: AttackDamage(5),
                attack_timer: AttackTimer::new(4.),
                behavior: Behavior::FollowPlayer {
                    distance: TILE_SIZE.x * 6.,
                },
                hitpoints: ResourcePool::<Health>::new(1),
                marker: Enemy,
                range: Range(TILE_SIZE.x * 12.),
                speed: Speed(2.),
                sprite: SpriteSheetBundle {
                    sprite: TextureAtlasSprite::new(0),
                    texture_atlas: texture_atlas_handle.clone(),
                    transform: Transform::from_translation(translation),
                    ..default()
                },
                collider: Collider::cuboid(HALF_TILE_SIZE.x, HALF_TILE_SIZE.y),
                render_layers: RenderLayers::layer(GROUND_LAYER),
                rigid_body: RigidBody::Dynamic,
                collision_groups: CollisionGroups::new(
                    ENEMY_GROUP,
                    ENEMY_GROUP | BUILDING_GROUP | FIRE_BREATH_GROUP,
                ),
            });

            enemy_entity_commands.insert((InGameEntity, LockedAxes::ROTATION_LOCKED, YSorted));
        }
    }
}

fn handle_enemy_behavior(
    mut enemy_query: Query<(&mut Transform, &Speed, &Behavior), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
) {
    let player_transform = player_query.single();
    let player_position = player_transform.translation.truncate();

    for (mut enemy_transform, enemy_speed, enemy_behavior) in &mut enemy_query {
        match enemy_behavior {
            &Behavior::FollowPlayer { distance } => {
                let enemy_position = enemy_transform.translation.truncate();
                if enemy_position.distance(player_position) > distance {
                    let enemy_direction = (player_position - enemy_position).normalize();
                    enemy_transform.translation.x += enemy_direction.x * enemy_speed.0;
                    enemy_transform.translation.y += enemy_direction.y * enemy_speed.0;
                }
            }
        }
    }
}

fn handle_enemy_attacks(
    mut spawn_projectile_event_writer: EventWriter<SpawnProjectileEvent>,
    mut enemy_query: Query<
        (Entity, &Transform, &mut AttackTimer, &Range, &AttackDamage),
        With<Enemy>,
    >,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let player_transform = player_query.single();
    let player_position = player_transform.translation.truncate();

    for (enemy_entity, enemy_transform, mut enemy_attack_timer, enemy_range, enemy_attack_damage) in
        &mut enemy_query
    {
        if enemy_attack_timer.tick(time.delta()).just_finished() {
            let enemy_position = enemy_transform.translation.truncate();

            if enemy_position.distance(player_position) <= enemy_range.0 {
                let direction = (player_position - enemy_position).normalize();
                let emitter = enemy_entity;

                spawn_projectile_event_writer.send(SpawnProjectileEvent::new(
                    enemy_attack_damage.0,
                    direction,
                    emitter,
                    enemy_position,
                    1000.,
                ))
            }
        }
    }
}
