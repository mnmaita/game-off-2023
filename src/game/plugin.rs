use bevy::{app::PluginGroupBuilder, prelude::*};

use super::{
    combat::CombatPlugin, enemy::EnemyPlugin, fire_breath::FireBreathPlugin,
    game_over::GameOverPlugin, hud::HudPlugin, level::LevelPlugin, player::PlayerPlugin,
    power_up::PowerUpSystemPlugin, score_system::ScoreSystemPlugin,
};

pub struct GamePlugin;

impl PluginGroup for GamePlugin {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CombatPlugin)
            .add(EnemyPlugin)
            .add(FireBreathPlugin)
            .add(GameOverPlugin)
            .add(HudPlugin)
            .add(LevelPlugin)
            .add(PlayerPlugin)
            .add(PowerUpSystemPlugin)
            .add(ScoreSystemPlugin)
    }
}
