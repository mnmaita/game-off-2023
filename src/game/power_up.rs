use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::{
    dynamics::{LockedAxes, RigidBody},
    geometry::{Collider, CollisionGroups, Sensor},
    plugin::RapierContext,
};
use rand::Rng;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    camera::{RenderLayer, YSorted},
    playing, AppState,
};

use super::{
    plugin::InGameEntity,
    resource_pool::{Health, ResourcePool},
    Player, HALF_TILE_SIZE, PLAYER_GROUP, POWERUP_GROUP,
};

pub(super) struct PowerUpSystemPlugin;

impl Plugin for PowerUpSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PowerUpEvent>();
        app.add_systems(OnEnter(AppState::InGame), load_scale_atlas);
        app.add_systems(
            FixedUpdate,
            (spawn_powerups, consume_powerups).run_if(playing()),
        );
    }
}

//event and event type
#[derive(Event)]
pub struct PowerUpEvent {
    transform: Transform,
    powerup_event_type: PowerUpEventType,
}
#[derive(Component)]
pub enum PowerUpEventType {
    HealingScale,
}

impl PowerUpEvent {
    pub fn new(transform: Transform, powerup_event_type: PowerUpEventType) -> Self {
        Self {
            transform,
            powerup_event_type,
        }
    }
}

#[derive(Bundle)]
pub struct PowerUpBundle {
    pub marker: PowerUp,
    pub animation_indices: AnimationIndices,
    pub animation_timer: AnimationTimer,
    pub sprite: SpriteSheetBundle,
    pub collider: Collider,
    pub render_layers: RenderLayers,
    pub sensor: Sensor,
    pub collision_groups: CollisionGroups,
}

#[derive(Component)]
pub struct PowerUp;

#[derive(Resource)]
pub struct ScaleTextureAtlasHandler(Handle<TextureAtlas>);

fn load_scale_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_healing_scale = asset_server
        .get_handle("textures/scale_anim.png")
        .unwrap_or_default();

    let texture_atlas_healing_scale =
        TextureAtlas::from_grid(texture_healing_scale, Vec2::new(40., 40.), 2, 1, None, None);
    let texture_atlas_handle_healing_scale = asset_server.add(texture_atlas_healing_scale);

    commands.insert_resource(ScaleTextureAtlasHandler(texture_atlas_handle_healing_scale));
}

fn spawn_powerups(
    mut commands: Commands,
    scale_texture_atlas_handler: Res<ScaleTextureAtlasHandler>,
    mut powerup_event_reader: EventReader<PowerUpEvent>,
) {
    for PowerUpEvent {
        transform,
        powerup_event_type,
    } in powerup_event_reader.read()
    {
        match powerup_event_type {
            PowerUpEventType::HealingScale => {
                // spawn powerup entity here with transform

                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.1) {
                    // spawneo
                    let mut powerup_entity_commands = commands.spawn(PowerUpBundle {
                        marker: PowerUp,
                        animation_indices: AnimationIndices::new(0, 1),
                        animation_timer: AnimationTimer::from_seconds(0.2),
                        sprite: SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(0),
                            texture_atlas: scale_texture_atlas_handler.0.clone(),
                            transform: *transform,
                            ..default()
                        },
                        collider: Collider::cuboid(HALF_TILE_SIZE.x, HALF_TILE_SIZE.y),
                        render_layers: RenderLayers::layer(RenderLayer::Sky.into()),
                        sensor: Sensor,
                        collision_groups: CollisionGroups::new(POWERUP_GROUP, PLAYER_GROUP),
                    });
                    powerup_entity_commands.insert((
                        InGameEntity,
                        LockedAxes::ROTATION_LOCKED,
                        YSorted,
                        RigidBody::Dynamic,
                    ));
                };
            }
        }
    }
}

fn consume_powerups(
    mut commands: Commands,
    powerup_query: Query<Entity, With<PowerUp>>,
    mut player_query: Query<&mut ResourcePool<Health>, With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    for entity in &powerup_query {
        for (entity1, entity2, intersecting) in rapier_context.intersections_with(entity) {
            if intersecting {
                if let Ok(mut hitpoints) = player_query.get_single_mut() {
                    hitpoints.add(50);
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}
