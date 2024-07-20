use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;
use rand::{seq::IteratorRandom, Rng};
use std::{collections::HashMap, time::Duration};

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    camera::{RenderLayer, YSorted},
    physics::Speed,
    playing, AppState,
};

use super::{
    combat::{AttackDamage, AttackTimer, Range, SpawnProjectileEvent},
    resource_pool::{Health, ResourcePool},
    BorderTile, Player, BUILDING_GROUP, ENEMY_GROUP, FIRE_BREATH_GROUP, HALF_TILE_SIZE, TILE_SIZE,
};

#[derive(Resource, Deref)]
struct SpriteAnimationMap(HashMap<SpriteAnimation, [usize; 2]>);

impl Default for SpriteAnimationMap {
    fn default() -> Self {
        Self(HashMap::from([
            (SpriteAnimation::RunLeft, [0, 7]),
            (SpriteAnimation::RunUpLeft, [16, 23]),
            (SpriteAnimation::RunUp, [32, 39]),
            (SpriteAnimation::RunUpRight, [48, 55]),
            (SpriteAnimation::RunRight, [64, 71]),
            (SpriteAnimation::RunDownRight, [80, 87]),
            (SpriteAnimation::RunDown, [96, 103]),
            (SpriteAnimation::RunDownLeft, [112, 119]),
            // TODO update all indexes
            (SpriteAnimation::AttackLeft, [12, 15]),
            (SpriteAnimation::AttackUpLeft, [28, 31]),
            (SpriteAnimation::AttackUp, [44, 47]),
            (SpriteAnimation::AttackUpRight, [60, 63]),
            (SpriteAnimation::AttackRight, [76, 79]),
            (SpriteAnimation::AttackDownRight, [92, 95]),
            (SpriteAnimation::AttackDown, [108, 111]),
            (SpriteAnimation::AttackDownLeft, [124, 127]),
        ]))
    }
}

pub(super) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer::new(3.));
        app.insert_resource(SpriteAnimationMap::default());

        app.add_systems(
            OnEnter(AppState::InGame),
            (load_atlas_handlers, setup_enemy_spawn_counter),
        );

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
    pub animation_indices: AnimationIndices,
    pub animation_timer: AnimationTimer,
    pub sprite_orientation: SpriteAnimation,
    pub sprite: SpriteBundle,
    pub texture_atlas: TextureAtlas,
    pub collider: Collider,
    pub render_layers: RenderLayers,
    pub rigid_body: RigidBody,
    pub collision_groups: CollisionGroups,
}

#[derive(Component, Eq, PartialEq, Hash, Debug, Copy, Clone)]
pub enum SpriteAnimation {
    RunLeft,
    RunUpLeft,
    RunUp,
    RunUpRight,
    RunRight,
    RunDownRight,
    RunDown,
    RunDownLeft,
    AttackLeft,
    AttackUpLeft,
    AttackUp,
    AttackUpRight,
    AttackRight,
    AttackDownRight,
    AttackDown,
    AttackDownLeft,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
struct EnemySpawnCounter(u32);

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

#[derive(Resource)]
pub struct TextureArcherAtlasHandle(Handle<TextureAtlasLayout>);

#[derive(Resource)]
pub struct TextureAxeAtlasHandle(Handle<TextureAtlasLayout>);

fn load_atlas_handlers(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_atlas_archer = TextureAtlasLayout::from_grid(UVec2::new(72, 78), 16, 8, None, None);
    let texture_atlas_handle_archer = asset_server.add(texture_atlas_archer);

    commands.insert_resource(TextureArcherAtlasHandle(texture_atlas_handle_archer));

    let texture_atlas_axe = TextureAtlasLayout::from_grid(UVec2::new(72, 78), 16, 8, None, None);
    let texture_atlas_handle_axe = asset_server.add(texture_atlas_axe);

    commands.insert_resource(TextureAxeAtlasHandle(texture_atlas_handle_axe));
}

fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    mut enemy_spawn_counter: ResMut<EnemySpawnCounter>,
    tile_query: Query<&Transform, With<BorderTile>>,
    texture_archer_atlas_handle: Res<TextureArcherAtlasHandle>,
    texture_axeman_atlas_handle: Res<TextureAxeAtlasHandle>,
) {
    let duration = enemy_spawn_timer.duration();

    if enemy_spawn_timer.tick(time.delta()).just_finished() {
        enemy_spawn_counter.0 = enemy_spawn_counter.0.wrapping_add(1);

        if enemy_spawn_counter.0 % 10 == 0 {
            enemy_spawn_timer.set_duration(Duration::from_secs_f32(
                1.0_f32.max(duration.as_secs_f32() - 0.5),
            ));
        }

        let mut rng = rand::thread_rng();
        if let Some(tile_transform) = tile_query.iter().choose(&mut rng) {
            let translation = tile_transform.translation.truncate().extend(1.);

            //pick a random texture atlas handle between archer and axe
            let (texture_atlas_handle, texture) = if rng.gen_bool(0.5) {
                (
                    texture_archer_atlas_handle.0.clone(),
                    asset_server
                        .get_handle("textures/enemy_archer.png")
                        .unwrap_or_default(),
                )
            } else {
                (
                    texture_axeman_atlas_handle.0.clone(),
                    asset_server
                        .get_handle("textures/enemy_axe.png")
                        .unwrap_or_default(),
                )
            };

            commands.spawn((
                EnemyBundle {
                    attack_damage: AttackDamage(5),
                    attack_timer: AttackTimer::new(3.),
                    behavior: Behavior::FollowPlayer {
                        distance: TILE_SIZE.x * 6.,
                    },
                    hitpoints: ResourcePool::<Health>::new(1),
                    marker: Enemy,
                    range: Range(TILE_SIZE.x * 15.),
                    speed: Speed(2.),
                    animation_indices: AnimationIndices::new(4, 11),
                    animation_timer: AnimationTimer::from_seconds(0.2),
                    sprite_orientation: SpriteAnimation::RunLeft,
                    sprite: SpriteBundle {
                        texture,
                        transform: Transform::from_translation(translation),
                        ..default()
                    },
                    texture_atlas: TextureAtlas {
                        layout: texture_atlas_handle,
                        index: 4,
                    },
                    collider: Collider::cuboid(HALF_TILE_SIZE.x, HALF_TILE_SIZE.y),
                    render_layers: RenderLayers::layer(RenderLayer::Ground.into()),
                    rigid_body: RigidBody::Dynamic,
                    collision_groups: CollisionGroups::new(
                        ENEMY_GROUP,
                        ENEMY_GROUP | BUILDING_GROUP | FIRE_BREATH_GROUP,
                    ),
                },
                StateScoped(AppState::GameOver),
                LockedAxes::ROTATION_LOCKED,
                YSorted,
            ));
        }
    }
}

fn setup_enemy_spawn_counter(mut commands: Commands) {
    commands.insert_resource(EnemySpawnCounter(0));
}

fn handle_enemy_behavior(
    mut enemy_query: Query<
        (
            &mut Transform,
            &Speed,
            &Behavior,
            &mut SpriteAnimation,
            &mut AnimationIndices,
            &mut TextureAtlas,
        ),
        With<Enemy>,
    >,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    sprite_animation_map: Res<SpriteAnimationMap>,
) {
    let player_transform = player_query.single();
    let player_position = player_transform.translation.truncate();

    for (
        mut enemy_transform,
        enemy_speed,
        enemy_behavior,
        mut sprite_orientation,
        mut animation_indices,
        mut texture_atlas,
    ) in &mut enemy_query
    {
        match enemy_behavior {
            &Behavior::FollowPlayer { distance } => {
                let enemy_position = enemy_transform.translation.truncate();

                let enemy_direction = (player_position - enemy_position).normalize();
                let old_sprite_orientation = *sprite_orientation;

                if enemy_position.distance(player_position) > distance {
                    enemy_transform.translation.x += enemy_direction.x * enemy_speed.0;
                    enemy_transform.translation.y += enemy_direction.y * enemy_speed.0;

                    // determine enemy quadrant based on enemy_direction
                    // and set sprite orientation accordingly
                    if enemy_direction.x == 0. && enemy_direction.y > 0. {
                        *sprite_orientation = SpriteAnimation::RunUp;
                    } else if enemy_direction.x == 0. && enemy_direction.y < 0. {
                        *sprite_orientation = SpriteAnimation::RunDown;
                    } else if enemy_direction.x > 0. && enemy_direction.y == 0. {
                        *sprite_orientation = SpriteAnimation::RunRight;
                    } else if enemy_direction.x < 0. && enemy_direction.y == 0. {
                        *sprite_orientation = SpriteAnimation::RunLeft;
                    } else if enemy_direction.x > 0. && enemy_direction.y > 0. {
                        *sprite_orientation = SpriteAnimation::RunUpRight;
                    } else if enemy_direction.x > 0. && enemy_direction.y < 0. {
                        *sprite_orientation = SpriteAnimation::RunDownRight;
                    } else if enemy_direction.x < 0. && enemy_direction.y > 0. {
                        *sprite_orientation = SpriteAnimation::RunUpLeft;
                    } else if enemy_direction.x < 0. && enemy_direction.y < 0. {
                        *sprite_orientation = SpriteAnimation::RunDownLeft;
                    }
                } else if enemy_direction.x == 0. && enemy_direction.y > 0. {
                    *sprite_orientation = SpriteAnimation::AttackUp;
                } else if enemy_direction.x == 0. && enemy_direction.y < 0. {
                    *sprite_orientation = SpriteAnimation::AttackDown;
                } else if enemy_direction.x > 0. && enemy_direction.y == 0. {
                    *sprite_orientation = SpriteAnimation::AttackRight;
                } else if enemy_direction.x < 0. && enemy_direction.y == 0. {
                    *sprite_orientation = SpriteAnimation::AttackLeft;
                } else if enemy_direction.x > 0. && enemy_direction.y > 0. {
                    *sprite_orientation = SpriteAnimation::AttackUpRight;
                } else if enemy_direction.x > 0. && enemy_direction.y < 0. {
                    *sprite_orientation = SpriteAnimation::AttackDownRight;
                } else if enemy_direction.x < 0. && enemy_direction.y > 0. {
                    *sprite_orientation = SpriteAnimation::AttackUpLeft;
                } else if enemy_direction.x < 0. && enemy_direction.y < 0. {
                    *sprite_orientation = SpriteAnimation::AttackDownLeft;
                }

                // if sprite orientation changed, update animation indices and sprite index
                if old_sprite_orientation != *sprite_orientation {
                    if let Some(value) = sprite_animation_map.get(&sprite_orientation) {
                        *animation_indices = AnimationIndices::new(value[0], value[1]);
                        texture_atlas.index = value[0];
                    };
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
                    800.,
                ));
            }
        }
    }
}
