use bevy::{
    color::palettes::css::{GOLD, LIMEGREEN, RED},
    prelude::*,
    text::BreakLineOn,
};

use crate::{playing, AppState};

use super::{
    resource_pool::{Fire, Health, ResourcePool},
    score_system::Score,
    Player,
};

const BAR_WIDTH: f32 = 150.;
const BAR_HEIGHT: f32 = 15.;
const BAR_BORDER_SIZE: f32 = 2.;

pub(super) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_hud);

        app.add_systems(
            FixedUpdate,
            (
                update_health_bar_display,
                update_fire_bar_display,
                update_score_display,
            )
                .run_if(playing()),
        );
    }
}

#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct FireBreathBar;

#[derive(Component)]
struct ScoreDisplay;

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            StateScoped(AppState::GameOver),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::End,
                    padding: UiRect::all(Val::Px(16.)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    border_color: BorderColor(Color::BLACK),
                    style: Style {
                        border: UiRect::all(Val::Px(BAR_BORDER_SIZE)),
                        width: Val::Px(BAR_WIDTH),
                        height: Val::Px(BAR_HEIGHT),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|health_bar_builder| {
                    health_bar_builder.spawn((
                        NodeBundle {
                            background_color: RED.into(),
                            style: Style {
                                width: Val::Px(BAR_WIDTH - BAR_BORDER_SIZE * 2.),
                                height: Val::Px(BAR_HEIGHT - BAR_BORDER_SIZE * 2.),
                                ..default()
                            },
                            ..default()
                        },
                        HealthBar,
                    ));
                });

            builder
                .spawn(NodeBundle {
                    border_color: BorderColor(Color::BLACK),
                    style: Style {
                        border: UiRect::all(Val::Px(BAR_BORDER_SIZE)),
                        width: Val::Px(BAR_WIDTH),
                        height: Val::Px(BAR_HEIGHT),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|fire_breath_bar_builder| {
                    fire_breath_bar_builder.spawn((
                        NodeBundle {
                            background_color: LIMEGREEN.into(),
                            style: Style {
                                width: Val::Px(BAR_WIDTH - BAR_BORDER_SIZE * 2.),
                                height: Val::Px(BAR_HEIGHT - BAR_BORDER_SIZE * 2.),
                                ..default()
                            },
                            ..default()
                        },
                        FireBreathBar,
                    ));
                });
        });

    // Score text in bottom middle of screen
    commands.spawn((
        StateScoped(AppState::GameOver),
        ScoreDisplay,
        TextBundle {
            text: Text {
                linebreak_behavior: BreakLineOn::NoWrap,
                sections: vec![TextSection {
                    value: "Score: 0".to_string(),
                    style: TextStyle {
                        font: asset_server
                            .get_handle("fonts/Prince Valiant.ttf")
                            .unwrap_or_default(),
                        font_size: 40.0,
                        color: GOLD.into(),
                    },
                }],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(5.),
                height: Val::Px(5.),
                ..default()
            },
            ..default()
        },
    ));
}

fn update_health_bar_display(
    player_query: Query<&ResourcePool<Health>, (Changed<ResourcePool<Health>>, With<Player>)>,
    mut health_bar_query: Query<&mut Style, With<HealthBar>>,
) {
    if let Ok(hitpoints) = player_query.get_single() {
        let mut style = health_bar_query.single_mut();

        style.width = Val::Px(BAR_WIDTH * hitpoints.current_percentage());
    }
}

fn update_fire_bar_display(
    player_query: Query<&ResourcePool<Fire>, (Changed<ResourcePool<Fire>>, With<Player>)>,
    mut fire_bar_query: Query<&mut Style, With<FireBreathBar>>,
) {
    if let Ok(fire_breath_resource) = player_query.get_single() {
        let mut style = fire_bar_query.single_mut();

        style.width = Val::Px(BAR_WIDTH * fire_breath_resource.current_percentage());
    }
}

fn update_score_display(
    player_query: Query<&Score, (Changed<Score>, With<Player>)>,
    mut score_text_display_query: Query<&mut Text, With<ScoreDisplay>>,
) {
    if let Ok(score_system) = player_query.get_single() {
        let mut score_text = score_text_display_query.single_mut();
        score_text.sections[0].value = format!(
            "Score: {} - Multiplier x {}",
            score_system.current(),
            score_system.multiplier()
        );
    }
}
