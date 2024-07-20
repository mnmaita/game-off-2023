use bevy::{ecs::system::SystemParam, prelude::*, render::view::RenderLayers, sprite::Anchor};
use bevy_rapier2d::prelude::*;
use noise::{NoiseFn, Perlin};
use pathfinding::prelude::Matrix;
use rand::{random, seq::SliceRandom, Rng};

use crate::{
    audio::{PlayMusicEvent, PlaybackSettings},
    camera::{RenderLayer, YSorted, YSortedInverse},
    game::{
        BUILDING_GROUP, ENEMY_GROUP, FIRE_BREATH_GROUP, GRID_SIZE, HALF_GRID_SIZE, HALF_TILE_SIZE,
        TILE_SIZE,
    },
    AppState,
};

use super::{
    combat::{AttackDamage, AttackTimer, Range},
    resource_pool::{Health, ResourcePool},
    Enemy,
};

pub(super) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (
                generate_level_matrix,
                generate_tilemaps,
                (
                    spawn_level_tiles.after(generate_tilemaps),
                    spawn_buildings,
                    spawn_hills,
                    spawn_mountains,
                    spawn_waves,
                )
                    .after(generate_level_matrix),
                play_background_music,
            ),
        );
    }
}

fn generate_tilemaps(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tileset_ground_texture_atlas_layout =
        TextureAtlasLayout::from_grid(TILE_SIZE.as_uvec2(), 16, 18, None, None);
    let tileset_objects_texture_atlas =
        TextureAtlasLayout::from_grid(TILE_SIZE.as_uvec2(), 38, 14, None, None);

    commands.insert_resource(TilesetGroundTextureAtlasHandle(
        asset_server.add(tileset_ground_texture_atlas_layout),
    ));
    commands.insert_resource(TilesetObjectsTextureAtlasHandle(
        asset_server.add(tileset_objects_texture_atlas),
    ));
}

fn generate_level_matrix(mut commands: Commands) {
    const MAP_OFFSET_X: f64 = 0.;
    const MAP_OFFSET_Y: f64 = 0.;
    const MAP_SCALE: f64 = 20.;

    let seed = random();
    let perlin = Perlin::new(seed);
    let tile_count = Tile::_LAST as u8;
    let mut level_matrix = Matrix::new(GRID_SIZE.y as usize, GRID_SIZE.x as usize, Tile::_LAST);

    for y in 0..GRID_SIZE.y as usize {
        for x in 0..GRID_SIZE.x as usize {
            let point = [
                (x as f64 - MAP_OFFSET_X) / MAP_SCALE,
                (y as f64 - MAP_OFFSET_Y) / MAP_SCALE,
            ];
            let noise_value = perlin.get(point).clamp(0., 1.);
            let scaled_noise_value =
                (noise_value * tile_count as f64).clamp(0., tile_count as f64 - 1.);
            let int_noise_value = scaled_noise_value.floor() as u8;

            if let Some(tile) = level_matrix.get_mut((x, y)) {
                *tile = int_noise_value.into();
            }
        }
    }

    commands.insert_resource(LevelMatrix(level_matrix));
}

fn spawn_level_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_matrix: Res<LevelMatrix>,
    tileset_ground_texture_atlas_layout_handle: Res<TilesetGroundTextureAtlasHandle>,
) {
    let tileset_ground_texture = asset_server
        .get_handle("textures/tileset_ground.png")
        .unwrap_or_default();

    for ((x, y), tile) in level_matrix.0.items() {
        let tile = *tile;
        let position = translate_grid_position_to_world_space(&(x, y));
        let translation = position.extend(0.0);
        let transform = Transform::from_translation(translation);
        let mut tile_entity = commands.spawn((
            TileBundle {
                render_layers: RenderLayers::layer(RenderLayer::Background.into()),
                sprite: SpriteBundle {
                    texture: tileset_ground_texture.clone(),
                    transform,
                    ..default()
                },
                texture_atlas: TextureAtlas {
                    layout: tileset_ground_texture_atlas_layout_handle.0.clone(),
                    index: tile.into(),
                },
                tile,
            },
            StateScoped(AppState::GameOver),
        ));

        if y == 0 || x == 0 || y == GRID_SIZE.y as usize - 1 || x == GRID_SIZE.x as usize - 1 {
            tile_entity.insert(BorderTile);
        }
    }
}

fn spawn_buildings(
    mut commands: Commands,
    level_matrix: Res<LevelMatrix>,
    asset_server: Res<AssetServer>,
) {
    const BUILDING_SPAWN_CHANCE: f32 = 0.01;

    let grass_tiles: Vec<Vec2> = level_matrix
        .items()
        .filter(|(_, tile)| **tile == Tile::Grass)
        .map(|(pos, _)| translate_grid_position_to_world_space(&pos))
        .collect();
    let total_buildings = (grass_tiles.len() as f32 * BUILDING_SPAWN_CHANCE).ceil() as usize;
    let mut rng = rand::thread_rng();
    let random_spawn_points = grass_tiles.choose_multiple(&mut rng, total_buildings);
    let building_tile_variants = [
        Rect::from_corners(Vec2::new(352., 96.), Vec2::new(400., 144.)),
        Rect::from_corners(Vec2::new(400., 96.), Vec2::new(448., 144.)),
    ];
    let texture = asset_server
        .get_handle("textures/tileset_objects.png")
        .unwrap_or_default();

    for position in random_spawn_points {
        let translation = position.extend(1.);

        commands.spawn((
            BuildingBundle {
                active_collision_types: ActiveCollisionTypes::all(),
                attack_damage: AttackDamage(5),
                attack_timer: AttackTimer::new(4.),
                collider: Collider::ball(HALF_TILE_SIZE.x),
                collision_groups: CollisionGroups::new(
                    BUILDING_GROUP,
                    ENEMY_GROUP | FIRE_BREATH_GROUP,
                ),
                hitpoints: ResourcePool::<Health>::new(1000),
                marker: Enemy,
                range: Range(TILE_SIZE.x * 20.),
                render_layers: RenderLayers::layer(RenderLayer::Ground.into()),
                rigid_body: RigidBody::Fixed,
                sprite: SpriteBundle {
                    sprite: Sprite {
                        flip_x: rng.gen_bool(0.5),
                        rect: Some(*building_tile_variants.choose(&mut rng).unwrap()),
                        ..default()
                    },
                    texture: texture.clone(),
                    transform: Transform::from_translation(translation),
                    ..default()
                },
            },
            StateScoped(AppState::GameOver),
            YSorted,
        ));
    }
}

fn spawn_hills(
    mut commands: Commands,
    level_matrix: Res<LevelMatrix>,
    asset_server: Res<AssetServer>,
) {
    const POSITION_OFFSET_FACTOR: f32 = 15.;

    let hill_tiles: Vec<Vec2> = level_matrix
        .items()
        .filter(|(_, tile)| **tile == Tile::Hills)
        .map(|(pos, _)| translate_grid_position_to_world_space(&pos))
        .collect();
    let mut rng = rand::thread_rng();
    let hill_tile_variants = [
        Rect::from_corners(Vec2::new(320., 64.), Vec2::new(368., 96.)),
        Rect::from_corners(Vec2::new(368., 64.), Vec2::new(400., 96.)),
        Rect::from_corners(Vec2::new(400., 80.), Vec2::new(432., 96.)),
        Rect::from_corners(Vec2::new(432., 80.), Vec2::new(448., 96.)),
    ];
    let texture = asset_server
        .get_handle("textures/tileset_objects.png")
        .unwrap_or_default();

    for position in hill_tiles {
        let position_offset = Vec2::new(
            rng.gen::<f32>() * POSITION_OFFSET_FACTOR,
            -HALF_TILE_SIZE.y + rng.gen::<f32>() * POSITION_OFFSET_FACTOR,
        );
        let translation = (position + position_offset).extend(1.);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::BottomCenter,
                    flip_x: rng.gen_bool(0.2),
                    rect: Some(*hill_tile_variants.choose(&mut rng).unwrap()),
                    ..default()
                },
                texture: texture.clone(),
                transform: Transform::from_translation(translation),
                ..default()
            },
            RenderLayers::layer(RenderLayer::Topography.into()),
            StateScoped(AppState::GameOver),
            YSorted,
        ));
    }
}

fn spawn_mountains(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_matrix: Res<LevelMatrix>,
) {
    const MOUNTAIN_TILE_SIZE: Vec2 = Vec2::new(64., 48.);
    const POSITION_OFFSET_FACTOR: f32 = 20.;

    let mountain_tiles: Vec<Vec2> = level_matrix
        .items()
        .filter(|(_, tile)| **tile == Tile::Mountains)
        .map(|(pos, _)| translate_grid_position_to_world_space(&pos))
        .collect();
    let mut rng = rand::thread_rng();
    let mountain_tile_variants = [
        Rect::from_corners(Vec2::ZERO, MOUNTAIN_TILE_SIZE),
        Rect::from_corners(
            Vec2::X * MOUNTAIN_TILE_SIZE.x,
            MOUNTAIN_TILE_SIZE + (Vec2::X * MOUNTAIN_TILE_SIZE.x),
        ),
        Rect::from_corners(
            Vec2::Y * MOUNTAIN_TILE_SIZE.y,
            MOUNTAIN_TILE_SIZE + (Vec2::Y * MOUNTAIN_TILE_SIZE.y),
        ),
        Rect::from_corners(MOUNTAIN_TILE_SIZE, MOUNTAIN_TILE_SIZE * 2.),
    ];
    let texture = asset_server
        .get_handle("textures/tileset_objects.png")
        .unwrap_or_default();

    for position in mountain_tiles {
        let position_offset = Vec2::new(
            rng.gen::<f32>() * POSITION_OFFSET_FACTOR,
            -MOUNTAIN_TILE_SIZE.y / 2. + rng.gen::<f32>() * POSITION_OFFSET_FACTOR,
        );
        let translation = (position + position_offset).extend(1.);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomCenter,
                    flip_x: rng.gen_bool(0.3),
                    rect: Some(*mountain_tile_variants.choose(&mut rng).unwrap()),
                    ..default()
                },
                texture: texture.clone(),
                transform: Transform::from_translation(translation),
                ..default()
            },
            RenderLayers::layer(RenderLayer::Topography.into()),
            StateScoped(AppState::GameOver),
            YSortedInverse,
        ));
    }
}

fn spawn_waves(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_matrix: Res<LevelMatrix>,
) {
    const WAVE_TILE_SIZE: Vec2 = Vec2::new(32., 16.);
    const POSITION_OFFSET_FACTOR: f32 = 8.;

    let water_tiles: Vec<Vec2> = level_matrix
        .items()
        .filter(|(_, tile)| **tile == Tile::Water)
        .map(|(pos, _)| translate_grid_position_to_world_space(&pos))
        .collect();
    let mut rng = rand::thread_rng();
    let wave_tiles =
        water_tiles.choose_multiple(&mut rng, (water_tiles.len() as f32 * 0.05) as usize);
    let texture = asset_server
        .get_handle("textures/tileset_objects.png")
        .unwrap_or_default();

    for position in wave_tiles {
        let position_offset = Vec2::new(
            rng.gen::<f32>() * POSITION_OFFSET_FACTOR,
            -WAVE_TILE_SIZE.y / 2. + rng.gen::<f32>() * POSITION_OFFSET_FACTOR,
        );
        let translation = (*position + position_offset).extend(2.);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomCenter,
                    flip_x: rng.gen_bool(0.3),
                    rect: Some(Rect::from_corners(
                        Vec2::new(208., 176.),
                        Vec2::new(240., 192.),
                    )),
                    ..default()
                },
                texture: texture.clone(),
                transform: Transform::from_translation(translation),
                ..default()
            },
            RenderLayers::layer(RenderLayer::Background.into()),
            StateScoped(AppState::GameOver),
            YSortedInverse,
        ));
    }
}

fn play_background_music(mut play_music_event_writer: EventWriter<PlayMusicEvent>) {
    play_music_event_writer.send(PlayMusicEvent::new(
        "theme2.ogg",
        Some(PlaybackSettings {
            loop_from: Some(0.0),
            volume: 0.25,
            ..default()
        }),
        None,
    ));
}

#[derive(Resource, Deref)]
pub struct TilesetGroundTextureAtlasHandle(Handle<TextureAtlasLayout>);

#[derive(Resource, Deref)]
pub struct TilesetObjectsTextureAtlasHandle(Handle<TextureAtlasLayout>);

#[derive(Resource, Deref)]
pub struct LevelMatrix(Matrix<Tile>);

#[derive(Bundle)]
pub struct BuildingBundle {
    pub active_collision_types: ActiveCollisionTypes,
    pub attack_damage: AttackDamage,
    pub attack_timer: AttackTimer,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub hitpoints: ResourcePool<Health>,
    pub marker: Enemy,
    pub range: Range,
    pub sprite: SpriteBundle,
    pub render_layers: RenderLayers,
    pub rigid_body: RigidBody,
}

#[derive(Component)]
pub struct BorderTile;

#[derive(Bundle)]
pub struct TileBundle {
    pub render_layers: RenderLayers,
    pub sprite: SpriteBundle,
    pub texture_atlas: TextureAtlas,
    pub tile: Tile,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
    Water,
    Sand,
    Grass,
    Hills,
    Mountains,
    _LAST,
}

impl From<u8> for Tile {
    fn from(value: u8) -> Self {
        // For every new type added to the enum, a new match arm should be added here.
        match value {
            0 => Self::Water,
            1 => Self::Sand,
            2 => Self::Grass,
            3 => Self::Hills,
            4 => Self::Mountains,
            #[cfg(debug_assertions)]
            _ => panic!("From<u8> for Tile: Missing match arm!"),
            #[cfg(not(debug_assertions))]
            _ => Self::Water,
        }
    }
}

impl From<Tile> for usize {
    fn from(value: Tile) -> Self {
        match value {
            Tile::Grass => 34,
            Tile::Hills => 242,
            Tile::Mountains => 242,
            Tile::Sand => 183,
            Tile::Water => {
                if random::<f32>() > 0.1 {
                    145
                } else {
                    let mut rng = rand::thread_rng();
                    *[146_usize, 147, 148].choose(&mut rng).unwrap()
                }
            }
            Tile::_LAST => 0,
        }
    }
}

#[derive(SystemParam)]
pub struct TileQuery<'w, 's> {
    tile_query: Query<'w, 's, (&'static Tile, &'static Transform)>,
}

impl<'w, 's> TileQuery<'w, 's> {
    pub fn get_from_position(&self, pos: Vec2) -> Option<&Tile> {
        self.tile_query
            .iter()
            .find(|(_, transform)| {
                let pos_transform = &Transform::from_translation(pos.extend(0.));
                translate_transform_to_grid_space(transform)
                    == translate_transform_to_grid_space(pos_transform)
            })
            .map(|(tile, _)| tile)
    }
}

pub fn translate_transform_to_grid_space(transform: &Transform) -> (usize, usize) {
    let x = ((transform.translation.x / TILE_SIZE.x) + HALF_GRID_SIZE.x).round();
    let y = ((transform.translation.y / TILE_SIZE.y) + HALF_GRID_SIZE.y).round();
    if x >= 0.0 && y >= 0.0 {
        (x as usize, y as usize)
    } else {
        (0, 0)
    }
}

pub fn translate_grid_position_to_world_space(pos: &(usize, usize)) -> Vec2 {
    let x = (pos.0 as f32 - HALF_GRID_SIZE.x) * TILE_SIZE.x;
    let y = (pos.1 as f32 - HALF_GRID_SIZE.y) * TILE_SIZE.y;
    Vec2::new(x, y)
}
