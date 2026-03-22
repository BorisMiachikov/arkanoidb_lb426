use bevy::prelude::*;

use crate::resources::game_state::GameState;
use crate::resources::score::{CurrentLevel, Lives, Score};

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

        // TODO (Этап 6): check_win_condition_system, check_ball_lost_system
        //   .run_if(in_state(GameState::Playing))
    }
}
