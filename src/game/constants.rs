use bevy::prelude::Vec2;
use bevy_rapier2d::prelude::Group;

pub const TILE_SIZE: Vec2 = Vec2::splat(16.);
pub const GRID_SIZE: Vec2 = Vec2::new(200., 200.);
pub const HALF_TILE_SIZE: Vec2 = Vec2::new(TILE_SIZE.x * 0.5, TILE_SIZE.y * 0.5);
pub const HALF_GRID_SIZE: Vec2 = Vec2::new(GRID_SIZE.x * 0.5, GRID_SIZE.y * 0.5);

pub const PLAYER_GROUP: Group = Group::GROUP_1;
pub const ENEMY_GROUP: Group = Group::GROUP_2;
pub const PROJECTILE_GROUP: Group = Group::GROUP_3;
pub const BUILDING_GROUP: Group = Group::GROUP_4;
pub const FIRE_BREATH_GROUP: Group = Group::GROUP_5;
pub const POWERUP_GROUP: Group = Group::GROUP_6;
