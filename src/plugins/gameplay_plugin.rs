use bevy::prelude::*;

use crate::resources::game_state::GameState;
use crate::resources::score::{CurrentLevel, Lives, Score};
use crate::systems::gameplay::{
    check_ball_lost_system, check_win_condition_system, handle_game_over_system,
    handle_level_complete_system,
};

/// Плагин: игровые правила, ресурсы, победа/поражение
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>();
        app.init_resource::<Lives>();
        app.init_resource::<CurrentLevel>();

        // Сразу переходим в Playing (до реализации главного меню в Этапе 12)
        app.add_systems(Startup, |mut next_state: ResMut<NextState<GameState>>| {
            next_state.set(GameState::Playing);
        });

        // Системы активной игры
        app.add_systems(
            Update,
            (check_ball_lost_system, check_win_condition_system)
                .run_if(in_state(GameState::Playing)),
        );

        // Обработка GameOver и LevelComplete
        app.add_systems(
            Update,
            handle_game_over_system.run_if(in_state(GameState::GameOver)),
        );
        app.add_systems(
            Update,
            handle_level_complete_system.run_if(in_state(GameState::LevelComplete)),
        );
    }
}
