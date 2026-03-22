use bevy::prelude::*;

use crate::components::ball::Ball;
use crate::components::brick::Brick;
use crate::components::velocity::Velocity;
use crate::resources::game_state::GameState;
use crate::resources::score::{Lives, Score};
use crate::setup::level::{
    BALL_INITIAL_VX, BALL_INITIAL_VY, BALL_SIZE, HALF_H, PADDLE_HEIGHT, PADDLE_Y,
};

/// Мяч упал за нижнюю границу — теряем жизнь и сбрасываем мяч.
/// При 0 жизней — переход в GameOver.
pub fn check_ball_lost_system(
    mut ball_query: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (mut transform, mut velocity) in &mut ball_query {
        if transform.translation.y < -(HALF_H + BALL_SIZE) {
            lives.count = lives.count.saturating_sub(1);

            if lives.count == 0 {
                next_state.set(GameState::GameOver);
            } else {
                // Сброс мяча над ракеткой
                transform.translation.x = 0.0;
                transform.translation.y = PADDLE_Y + PADDLE_HEIGHT + BALL_SIZE;
                velocity.x = BALL_INITIAL_VX;
                velocity.y = BALL_INITIAL_VY;
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

/// В состоянии GameOver: Enter/Space → рестарт
pub fn handle_game_over_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        score.value = 0;
        lives.count = 3;
        next_state.set(GameState::Playing);
    }
}

/// В состоянии LevelComplete: Enter/Space → следующий уровень (рестарт)
pub fn handle_level_complete_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}
