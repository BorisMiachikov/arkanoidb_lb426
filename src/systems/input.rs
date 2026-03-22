use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::paddle::Paddle;
use crate::components::velocity::Velocity;
use crate::resources::score::BallSpeedMultiplier;
use crate::setup::level::{BALL_INITIAL_VX, BALL_INITIAL_VY, BALL_SIZE, PADDLE_HEIGHT, PADDLE_Y};

const HALF_WINDOW_WIDTH: f32 = 400.0;
const WALL_THICKNESS: f32 = 16.0;
const PADDLE_HALF_WIDTH: f32 = 60.0;
const LEFT_BOUND: f32 = -HALF_WINDOW_WIDTH + WALL_THICKNESS + PADDLE_HALF_WIDTH;
const RIGHT_BOUND: f32 = HALF_WINDOW_WIDTH - WALL_THICKNESS - PADDLE_HALF_WIDTH;

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

/// Мяч прилипший к ракетке: следует за ней и запускается по Пробелу.
/// Направление запуска зависит от нажатой клавиши движения.
pub fn ball_stuck_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    speed_mul: Res<BallSpeedMultiplier>,
    mut ball_query: Query<(Entity, &mut Transform, &mut Velocity), (With<Ball>, With<BallStuck>)>,
    paddle_query: Query<&Transform, (With<Paddle>, Without<Ball>)>,
) {
    let Ok(paddle_tf) = paddle_query.get_single() else {
        return;
    };

    for (ball_entity, mut ball_tf, mut velocity) in &mut ball_query {
        // Мяч держится на центре ракетки
        ball_tf.translation.x = paddle_tf.translation.x;
        ball_tf.translation.y = PADDLE_Y + PADDLE_HEIGHT / 2.0 + BALL_SIZE / 2.0 + 1.0;

        // Запуск по Пробелу
        if keys.just_pressed(KeyCode::Space) {
            commands.entity(ball_entity).remove::<BallStuck>();

            let m = speed_mul.0;
            let vx = if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
                -BALL_INITIAL_VX * m
            } else if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
                BALL_INITIAL_VX * m
            } else {
                0.0
            };
            velocity.x = vx;
            velocity.y = BALL_INITIAL_VY * m;
        }
    }
}
