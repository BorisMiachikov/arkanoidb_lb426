use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::collider::Collider;
use crate::components::paddle::Paddle;
use crate::components::velocity::Velocity;
use crate::resources::score::BallSpeedMultiplier;
use crate::setup::level::{BALL_INITIAL_VX, BALL_INITIAL_VY, BALL_SIZE, HALF_W, PADDLE_HEIGHT, PADDLE_Y, WALL_THICKNESS};

/// Обработка ввода с клавиатуры — движение ракетки (A/D или ←/→)
pub fn paddle_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Paddle, &Collider, &mut Transform)>,
) {
    for (paddle, collider, mut transform) in &mut query {
        let mut direction = 0.0_f32;

        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }

        // Границы считаем по текущей ширине коллайдера (меняется при бонусе PaddleGrow)
        let half_w = collider.half_width;
        let left_bound  = -HALF_W + WALL_THICKNESS + half_w;
        let right_bound =  HALF_W - WALL_THICKNESS - half_w;

        let new_x = transform.translation.x + direction * paddle.speed * time.delta_secs();
        transform.translation.x = new_x.clamp(left_bound, right_bound);
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
