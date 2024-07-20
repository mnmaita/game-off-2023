use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    camera::{RenderLayer, YSorted},
    physics::Speed,
    AppState,
};

use super::{
    resource_pool::{Fire, Health, ResourcePool},
    score_system::Score,
    PLAYER_GROUP, POWERUP_GROUP, PROJECTILE_GROUP,
};

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_player);
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
    pub speed: Speed,
    pub marker: Player,
    pub render_layers: RenderLayers,
    pub sprite: SpriteBundle,
    pub texture_atlas: TextureAtlas,
}

#[derive(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server
        .get_handle("textures/dragon.png")
        .unwrap_or_default();
    let texture_atlas_layout =
        TextureAtlasLayout::from_grid(UVec2::new(191, 161), 12, 1, None, None);
    let texture_atlas_layout_handle = asset_server.add(texture_atlas_layout);

    commands.spawn((
        PlayerBundle {
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
            sprite: SpriteBundle {
                texture,
                transform: Transform::from_translation(Vec2::ONE.extend(1.)),
                ..default()
            },
            texture_atlas: texture_atlas_layout_handle.into(),
        },
        StateScoped(AppState::GameOver),
        YSorted,
    ));
}
