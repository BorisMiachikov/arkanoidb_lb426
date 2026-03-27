use bevy::prelude::*;

use crate::resources::game_state::GameState;
use crate::resources::score::{
    AudioSettings, BallSpeedMultiplier, CurrentLevel, DebugSkipPending, HighScore,
    Lives, MenuSelection, NameInput, OptionsSelection, Paused, Score, ScoreTable,
};
use crate::systems::gameplay::{
    check_ball_lost_system, check_win_condition_system, debug_auto_advance_system,
    debug_skip_level_system, handle_enter_name_system, handle_game_over_system,
    handle_highscores_system, handle_level_complete_system, handle_main_menu_system,
    handle_options_system, handle_pause_system, track_highscore_system,
};

/// Плагин: игровые правила, ресурсы, победа/поражение
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>();
        app.init_resource::<Lives>();
        app.init_resource::<CurrentLevel>();
        app.init_resource::<BallSpeedMultiplier>();
        app.init_resource::<DebugSkipPending>();
        app.init_resource::<Paused>();
        app.init_resource::<MenuSelection>();
        app.init_resource::<OptionsSelection>();
        app.init_resource::<AudioSettings>();
        app.init_resource::<NameInput>();
        app.insert_resource(ScoreTable::load());
        app.insert_resource(HighScore::load());

        // Главное меню → старт игры
        app.add_systems(
            Update,
            handle_main_menu_system.run_if(in_state(GameState::MainMenu)),
        );

        // Системы активной игры (не работают на паузе)
        app.add_systems(
            Update,
            (
                check_ball_lost_system,
                check_win_condition_system,
                debug_skip_level_system,
                track_highscore_system,
            )
                .run_if(in_state(GameState::Playing))
                .run_if(|p: Res<Paused>| !p.0),
        );

        // Пауза — только в Playing
        app.add_systems(
            Update,
            handle_pause_system.run_if(in_state(GameState::Playing)),
        );

        // Options
        app.add_systems(
            Update,
            handle_options_system.run_if(in_state(GameState::Options)),
        );

        // High Scores + Enter Name
        app.add_systems(
            Update,
            handle_highscores_system.run_if(in_state(GameState::HighScores)),
        );
        app.add_systems(
            Update,
            handle_enter_name_system.run_if(in_state(GameState::EnterName)),
        );

        // Обработка GameOver и LevelComplete
        app.add_systems(
            Update,
            handle_game_over_system.run_if(in_state(GameState::GameOver)),
        );
        app.add_systems(
            Update,
            (debug_auto_advance_system, handle_level_complete_system)
                .run_if(in_state(GameState::LevelComplete)),
        );
    }
}
