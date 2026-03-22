use bevy::prelude::*;

use crate::components::paddle::Paddle;

const HALF_WINDOW_WIDTH: f32 = 400.0;
const PADDLE_HALF_WIDTH: f32 = 60.0;
const LEFT_BOUND: f32 = -HALF_WINDOW_WIDTH + PADDLE_HALF_WIDTH;
const RIGHT_BOUND: f32 = HALF_WINDOW_WIDTH - PADDLE_HALF_WIDTH;

/// Обработка ввода с клавиатуры — движение ракетки (A/D или ←/→)
pub fn paddle_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    for (paddle, mut transform) in &mut query {
        let mut direction = 0.0_f32;

        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }

        let new_x = transform.translation.x + direction * paddle.speed * time.delta_secs();
        transform.translation.x = new_x.clamp(LEFT_BOUND, RIGHT_BOUND);
    }
}
