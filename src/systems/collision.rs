use bevy::prelude::*;

use crate::components::ball::Ball;
use crate::components::collider::Collider;
use crate::components::paddle::Paddle;
use crate::components::velocity::Velocity;
use crate::components::wall::Wall;

/// Сторона столкновения (с точки зрения мяча)
enum CollisionSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// AABB-столкновение двух прямоугольников.
/// Возвращает сторону, с которой столкнулся объект A (мяч).
fn aabb_collision(
    pos_a: Vec2,
    half_a: Vec2,
    pos_b: Vec2,
    half_b: Vec2,
) -> Option<CollisionSide> {
    let dx = pos_a.x - pos_b.x;
    let dy = pos_a.y - pos_b.y;
    let overlap_x = half_a.x + half_b.x - dx.abs();
    let overlap_y = half_a.y + half_b.y - dy.abs();

    if overlap_x > 0.0 && overlap_y > 0.0 {
        if overlap_x < overlap_y {
            if dx > 0.0 {
                Some(CollisionSide::Right)
            } else {
                Some(CollisionSide::Left)
            }
        } else if dy > 0.0 {
            Some(CollisionSide::Top)
        } else {
            Some(CollisionSide::Bottom)
        }
    } else {
        None
    }
}

/// Ball ↔ Wall: отскок от стен
pub fn ball_wall_collision_system(
    mut ball_query: Query<(&mut Velocity, &mut Transform, &Collider), With<Ball>>,
    wall_query: Query<(&Transform, &Collider), (With<Wall>, Without<Ball>)>,
) {
    for (mut velocity, mut ball_tf, ball_col) in &mut ball_query {
        let ball_pos = ball_tf.translation.truncate();
        let ball_half = Vec2::new(ball_col.half_width, ball_col.half_height);

        for (wall_tf, wall_col) in &wall_query {
            let wall_pos = wall_tf.translation.truncate();
            let wall_half = Vec2::new(wall_col.half_width, wall_col.half_height);

            if let Some(side) = aabb_collision(ball_pos, ball_half, wall_pos, wall_half) {
                match side {
                    CollisionSide::Left | CollisionSide::Right => {
                        velocity.x = -velocity.x;
                        // Корректируем позицию, чтобы мяч не застрял в стене
                        let overlap = ball_half.x + wall_half.x - (ball_pos.x - wall_pos.x).abs();
                        if ball_pos.x > wall_pos.x {
                            ball_tf.translation.x += overlap;
                        } else {
                            ball_tf.translation.x -= overlap;
                        }
                    }
                    CollisionSide::Top | CollisionSide::Bottom => {
                        velocity.y = -velocity.y;
                        let overlap = ball_half.y + wall_half.y - (ball_pos.y - wall_pos.y).abs();
                        if ball_pos.y > wall_pos.y {
                            ball_tf.translation.y += overlap;
                        } else {
                            ball_tf.translation.y -= overlap;
                        }
                    }
                }
            }
        }
    }
}

/// Ball ↔ Paddle: отскок от ракетки с угловым эффектом
pub fn ball_paddle_collision_system(
    mut ball_query: Query<(&mut Velocity, &mut Transform, &Collider), With<Ball>>,
    paddle_query: Query<(&Transform, &Collider), (With<Paddle>, Without<Ball>)>,
) {
    for (mut velocity, mut ball_tf, ball_col) in &mut ball_query {
        // Реагируем только если мяч летит вниз
        if velocity.y >= 0.0 {
            continue;
        }

        let ball_pos = ball_tf.translation.truncate();
        let ball_half = Vec2::new(ball_col.half_width, ball_col.half_height);

        for (paddle_tf, paddle_col) in &paddle_query {
            let paddle_pos = paddle_tf.translation.truncate();
            let paddle_half = Vec2::new(paddle_col.half_width, paddle_col.half_height);

            if let Some(side) = aabb_collision(ball_pos, ball_half, paddle_pos, paddle_half) {
                match side {
                    CollisionSide::Top | CollisionSide::Bottom => {
                        // Угол отскока зависит от точки попадания (-1..1)
                        let hit_factor =
                            (ball_pos.x - paddle_pos.x) / paddle_half.x;
                        let speed =
                            (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
                        velocity.x = hit_factor * speed * 0.75;
                        velocity.y = (speed * speed - velocity.x * velocity.x)
                            .abs()
                            .sqrt();

                        // Выталкиваем мяч над ракеткой
                        let overlap = ball_half.y + paddle_half.y
                            - (ball_pos.y - paddle_pos.y).abs();
                        ball_tf.translation.y += overlap;
                    }
                    CollisionSide::Left | CollisionSide::Right => {
                        velocity.x = -velocity.x;
                    }
                }
            }
        }
    }
}
