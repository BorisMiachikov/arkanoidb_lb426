use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::brick::Brick;
use crate::components::velocity::Velocity;
use crate::resources::game_state::GameState;
use crate::resources::level_data::LEVELS;
use crate::resources::score::{CurrentLevel, Lives, Score};
use crate::setup::level::HALF_H;

/// Мяч упал за нижнюю границу — теряем жизнь.
/// При наличии жизней — мяч прилипает к ракетке (BallStuck).
/// При 0 жизней — GameOver.
pub fn check_ball_lost_system(
    mut commands: Commands,
    mut ball_query: Query<(Entity, &mut Transform, &mut Velocity), With<Ball>>,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (ball_entity, mut transform, mut velocity) in &mut ball_query {
        if transform.translation.y < -(HALF_H + 20.0) {
            lives.count = lives.count.saturating_sub(1);

            if lives.count == 0 {
                next_state.set(GameState::GameOver);
            } else {
                velocity.x = 0.0;
                velocity.y = 0.0;
                transform.translation.x = 0.0;
                commands.entity(ball_entity).insert(BallStuck);
            }
        }
    }
}

/// Все блоки уничтожены — победа на уровне.
pub fn check_win_condition_system(
    brick_query: Query<Entity, With<Brick>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() == GameState::Playing && brick_query.is_empty() {
        next_state.set(GameState::LevelComplete);
    }
}

/// В состоянии GameOver: Enter/Space → рестарт с первого уровня
pub fn handle_game_over_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut current_level: ResMut<CurrentLevel>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        score.value = 0;
        lives.count = 3;
        current_level.number = 0;
        next_state.set(GameState::Playing);
    }
}

/// В состоянии LevelComplete: Enter/Space → следующий уровень
pub fn handle_level_complete_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        current_level.number += 1;
        // После последнего уровня — начинаем сначала
        if current_level.number as usize >= LEVELS.len() {
            current_level.number = 0;
        }
        next_state.set(GameState::Playing);
    }
}
