use bevy::{prelude::*, render::view::RenderLayers};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    camera::{YSorted, SKY_LAYER},
    AppState,
};

use super::{
    resource_pool::{Fire, Health, ResourcePool},
    score_system::Score,
    InGameEntity,
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
    pub marker: Player,
    pub render_layers: RenderLayers,
    pub spritesheet: SpriteSheetBundle,
}

#[derive(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server
        .get_handle("textures/dragon.png")
        .unwrap_or_default();
    let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(191., 161.), 12, 1, None, None);
    let texture_atlas_handle = asset_server.add(texture_atlas);

    let mut player_entity_commands = commands.spawn(PlayerBundle {
        animation_indices: AnimationIndices::new(0, 2),
        animation_timer: AnimationTimer::from_seconds(0.2),
        collider: Collider::ball(80.5),
        collision_groups: CollisionGroups::new(Group::GROUP_1, Group::GROUP_1 | Group::GROUP_3),
        fire_breath_resource: ResourcePool::<Fire>::new(100),
        hitpoints: ResourcePool::<Health>::new(100),
        score: Score::new(0, 1),
        marker: Player,
        render_layers: RenderLayers::layer(SKY_LAYER),
        spritesheet: SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_translation(Vec2::ZERO.extend(1.)),
            ..default()
        },
    });

    player_entity_commands.insert((InGameEntity, YSorted));
}
