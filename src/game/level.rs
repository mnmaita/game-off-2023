use bevy::{
    audio::{Volume, VolumeLevel},
    ecs::system::SystemParam,
    prelude::*,
    render::view::RenderLayers,
};
use noise::{NoiseFn, Perlin};
use rand::random;

use crate::{
    audio::PlayMusicEvent,
    camera::BACKGROUND_LAYER,
    game::{GRID_SIZE, HALF_GRID_SIZE, TILE_SIZE},
    AppState,
};

pub(super) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (generate_level, play_background_music).chain(),
        );
    }
}

fn generate_level(mut commands: Commands) {
    const MAP_OFFSET_X: f64 = 0.;
    const MAP_OFFSET_Y: f64 = 0.;
    const MAP_SCALE: f64 = 20.;

    let seed = random();
    let perlin = Perlin::new(seed);
    let tile_count = Tile::_LAST as u8;

    for y in 0..GRID_SIZE.y as i32 {
        for x in 0..GRID_SIZE.x as i32 {
            let point = [
                (x as f64 - MAP_OFFSET_X) / MAP_SCALE,
                (y as f64 - MAP_OFFSET_Y) / MAP_SCALE,
            ];
            let noise_value = perlin.get(point).clamp(0., 1.);
            let scaled_noise_value =
                (noise_value * tile_count as f64).clamp(0., tile_count as f64 - 1.);
            let int_noise_value = scaled_noise_value.floor() as u8;
            let tile: Tile = int_noise_value.into();
            let color = tile.into();
            let custom_size = Some(TILE_SIZE);
            let position = (Vec2::new(x as f32, y as f32) - HALF_GRID_SIZE) * TILE_SIZE;
            let translation = position.extend(0.0);
            let transform = Transform::from_translation(translation);

            let mut tile_entity = commands.spawn(TileBundle {
                render_layers: RenderLayers::layer(BACKGROUND_LAYER),
                sprite: SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size,
                        ..default()
                    },
                    transform,
                    ..default()
                },
                tile,
            });

            if y == 0 || x == 0 || y == GRID_SIZE.y as i32 - 1 || x == GRID_SIZE.x as i32 - 1 {
                tile_entity.insert(BorderTile);
            }
        }
    }
}

fn play_background_music(mut play_music_event_writer: EventWriter<PlayMusicEvent>) {
    play_music_event_writer.send(PlayMusicEvent::new(
        "theme2.ogg",
        Some(PlaybackSettings {
            volume: Volume::Absolute(VolumeLevel::new(0.25)),
            ..default()
        }),
        None,
    ));
}

#[derive(Component)]
pub struct BorderTile;

#[derive(Bundle)]
pub struct TileBundle {
    pub render_layers: RenderLayers,
    pub sprite: SpriteBundle,
    pub tile: Tile,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
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

impl From<Tile> for Color {
    fn from(value: Tile) -> Self {
        match value {
            Tile::Grass => Self::DARK_GREEN,
            Tile::Hills => Self::GRAY,
            Tile::Mountains => Self::DARK_GRAY,
            Tile::Water => Self::BLUE,
            Tile::Sand => Self::BEIGE,
            Tile::_LAST => Self::default(),
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
    let half_columns = GRID_SIZE.x * 0.5;
    let half_rows = GRID_SIZE.y * 0.5;
    let x = ((transform.translation.x / TILE_SIZE.x) + half_columns).round();
    let y = ((transform.translation.y / TILE_SIZE.y) + half_rows).round();
    if x >= 0.0 && y >= 0.0 {
        (x as usize, y as usize)
    } else {
        (0, 0)
    }
}
