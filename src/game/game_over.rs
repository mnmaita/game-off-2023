use bevy::prelude::*;

use crate::{
    audio::{PlayMusicEvent, PlaybackSettings},
    playing, AppState,
};

use super::{
    resource_pool::{Health, ResourcePool},
    score_system::Score,
    Player,
};

pub(super) struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, handle_game_over_button_interactions);

        app.add_systems(
            FixedUpdate,
            (
                check_game_over_condition.run_if(playing()),
                (fade_out_screen, fade_in_text).run_if(in_state(AppState::GameOver)),
            ),
        );

        app.add_systems(
            OnEnter(AppState::GameOver),
            (
                play_background_music,
                display_game_over_screen,
                update_score_display.after(display_game_over_screen),
            ),
        );
    }
}

#[derive(Component)]
enum GameOverButtonAction {
    BackToMenu,
}

#[derive(Component)]
struct GameOverBackground;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
struct ScoreDisplay;

fn check_game_over_condition(
    mut next_state: ResMut<NextState<AppState>>,
    query: Query<&ResourcePool<Health>, (With<Player>, Changed<ResourcePool<Health>>)>,
) {
    if let Ok(player_health) = query.get_single() {
        if player_health.current() == 0 {
            next_state.set(AppState::GameOver);
        }
    }
}

fn display_game_over_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            GameOverBackground,
            StateScoped(AppState::GameOver),
            NodeBundle {
                background_color: BackgroundColor(Color::BLACK.with_alpha(0.)),
                style: Style {
                    align_items: AlignItems::Center,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|builder| {
            builder.spawn((
                GameOverText,
                TextBundle {
                    text: Text::from_section(
                        "Game Over",
                        TextStyle {
                            color: Color::WHITE.with_alpha(0.),
                            font: asset_server
                                .get_handle("fonts/MorrisRomanAlternate-Black.ttf")
                                .unwrap_or_default(),
                            font_size: 64.,
                        },
                    ),
                    ..default()
                },
            ));

            builder.spawn((
                GameOverText,
                ScoreDisplay,
                TextBundle {
                    text: Text::from_section(
                        "Score:",
                        TextStyle {
                            color: Color::WHITE.with_alpha(0.),
                            font: asset_server
                                .get_handle("fonts/MorrisRomanAlternate-Black.ttf")
                                .unwrap_or_default(),
                            font_size: 32.,
                        },
                    ),
                    ..default()
                },
            ));

            builder
                .spawn((
                    ButtonBundle {
                        background_color: BackgroundColor(Color::default().with_alpha(0.)),
                        ..default()
                    },
                    GameOverButtonAction::BackToMenu,
                ))
                .with_children(|button| {
                    button.spawn((
                        GameOverText,
                        TextBundle {
                            text: Text::from_section(
                                "Back to Menu",
                                TextStyle {
                                    color: Color::WHITE.with_alpha(0.),
                                    font: asset_server
                                        .get_handle("fonts/MorrisRomanAlternate-Black.ttf")
                                        .unwrap_or_default(),
                                    font_size: 32.0,
                                },
                            ),
                            ..default()
                        },
                    ));
                });
        });
}

fn fade_out_screen(mut query: Query<&mut BackgroundColor, With<GameOverBackground>>) {
    let mut background_color = query.single_mut();

    if background_color.0.alpha() < 1. {
        let alpha = background_color.0.alpha();
        background_color.0.set_alpha(alpha + 0.01);
    }
}

fn update_score_display(
    mut score_display_query: Query<&mut Text, (With<GameOverText>, With<ScoreDisplay>)>,
    player_query: Query<&Score, With<Player>>,
) {
    if let Ok(player_score) = player_query.get_single() {
        let mut score_text = score_display_query.single_mut();
        score_text.sections[0].value = format!("Score: {}", player_score.current());
    }
}

fn fade_in_text(
    mut query: Query<&mut Text, With<GameOverText>>,
    background_query: Query<&BackgroundColor, With<GameOverBackground>>,
) {
    let background_color = background_query.single();
    for mut text in &mut query {
        if background_color.0.alpha() >= 0.5 && text.sections[0].style.color.alpha() < 1. {
            let alpha = text.sections[0].style.color.alpha();
            text.sections[0].style.color.set_alpha(alpha + 0.001);
        }
    }
}

fn play_background_music(mut play_music_event_writer: EventWriter<PlayMusicEvent>) {
    play_music_event_writer.send(PlayMusicEvent::new(
        "theme3.ogg",
        Some(PlaybackSettings {
            volume: 0.25,
            ..default()
        }),
        None,
    ));
}

fn handle_game_over_button_interactions(
    mut app_state: ResMut<NextState<AppState>>,
    query: Query<(&Interaction, &GameOverButtonAction), With<Button>>,
) {
    for (interaction, game_over_button_action) in query.iter() {
        match interaction {
            Interaction::Pressed => match game_over_button_action {
                GameOverButtonAction::BackToMenu => app_state.set(AppState::MainMenu),
            },
            Interaction::Hovered => (),
            Interaction::None => (),
        }
    }
}
