#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use animation::AnimationPlugin;
use audio::{audio_assets_loaded, AudioPlugin, BgmChannel};
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_kira_audio::{AudioChannel, AudioControl};
use camera::CameraPlugin;
use fonts::{font_assets_loaded, FontsPlugin};
use game::GamePlugin;
use input::InputPlugin;
use main_menu::MainMenuPlugin;
use physics::PhysicsPlugin;
use textures::{texture_assets_loaded, TexturesPlugin};

mod animation;
mod audio;
mod camera;
#[cfg(debug_assertions)]
mod debug;
mod fonts;
mod game;
mod input;
mod main_menu;
mod physics;
mod textures;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        EmbeddedAssetPlugin {
            mode: PluginMode::ReplaceDefault,
        },
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                mode: AssetMode::Unprocessed,
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "DragonSkale".into(),
                    ..default()
                }),
                ..default()
            }),
        AnimationPlugin,
        AudioPlugin,
        CameraPlugin,
        #[cfg(debug_assertions)]
        debug::DebugPlugin,
        FontsPlugin,
        GamePlugin,
        InputPlugin,
        MainMenuPlugin,
        PhysicsPlugin,
        TexturesPlugin,
    ));

    app.init_state::<AppState>();

    app.enable_state_scoped_entities::<AppState>();

    app.insert_resource(Msaa::Off);

    app.insert_resource(ClearColor(Color::srgb(0., 0., 0.)));

    app.add_systems(
        Update,
        handle_asset_load.run_if(assets_loaded().and_then(run_once())),
    );

    app.add_systems(
        Update,
        stop_music_on_transition.run_if(state_changed::<AppState>),
    );

    app.run();
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, States)]
pub enum AppState {
    #[default]
    Setup,
    MainMenu,
    InGame,
    GameOver,
}

fn handle_asset_load(mut state: ResMut<NextState<AppState>>) {
    #[cfg(debug_assertions)]
    info!("Assets loaded successfully.");
    state.set(AppState::MainMenu);
}

pub fn stop_music_on_transition(bgm_audio_channel: Res<AudioChannel<BgmChannel>>) {
    bgm_audio_channel.stop();
}

pub fn playing() -> impl Condition<()> {
    IntoSystem::into_system(in_state(AppState::InGame))
}

fn assets_loaded() -> impl Condition<()> {
    texture_assets_loaded()
        .and_then(audio_assets_loaded())
        .and_then(font_assets_loaded())
}
