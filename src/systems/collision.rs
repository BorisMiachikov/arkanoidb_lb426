use bevy::prelude::*;

use crate::components::ball::Ball;
use crate::components::brick::Brick;
use crate::components::collider::Collider;
use crate::components::paddle::Paddle;
use crate::components::velocity::Velocity;
use crate::components::wall::Wall;
use crate::resources::score::Score;

/// Сторона столкновения (с точки зрения мяча)
enum CollisionSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// AABB-столкновение двух прямоугольников.
/// Возвращает сторону, с которой столкнулся объект A.
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

/// Ball ↔ Wall: отскок от стен.
/// Позиция мяча обновляется после каждой коррекции.
pub fn ball_wall_collision_system(
    mut ball_query: Query<(&mut Velocity, &mut Transform, &Collider), With<Ball>>,
    wall_query: Query<(&Transform, &Collider), (With<Wall>, Without<Ball>)>,
) {
    for (mut velocity, mut ball_tf, ball_col) in &mut ball_query {
        let ball_half = Vec2::new(ball_col.half_width, ball_col.half_height);

        for (wall_tf, wall_col) in &wall_query {
            // Читаем актуальную позицию мяча на каждой итерации
            let ball_pos = ball_tf.translation.truncate();
            let wall_pos = wall_tf.translation.truncate();
            let wall_half = Vec2::new(wall_col.half_width, wall_col.half_height);

            if let Some(side) = aabb_collision(ball_pos, ball_half, wall_pos, wall_half) {
                match side {
                    CollisionSide::Left | CollisionSide::Right => {
                        velocity.x = -velocity.x;
                        let overlap =
                            ball_half.x + wall_half.x - (ball_pos.x - wall_pos.x).abs();
                        if ball_pos.x > wall_pos.x {
                            ball_tf.translation.x += overlap;
                        } else {
                            ball_tf.translation.x -= overlap;
                        }
                    }
                    CollisionSide::Top | CollisionSide::Bottom => {
                        velocity.y = -velocity.y;
                        let overlap =
                            ball_half.y + wall_half.y - (ball_pos.y - wall_pos.y).abs();
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

/// Ball ↔ Brick: разрушение блоков и отскок.
/// Здоровье прочных блоков уменьшается, нулевые — удаляются.
/// Скорость отражается один раз за кадр.
pub fn ball_brick_collision_system(
    mut commands: Commands,
    mut ball_query: Query<(&mut Velocity, &Transform, &Collider), With<Ball>>,
    mut brick_query: Query<(Entity, &Transform, &Collider, &mut Brick)>,
    mut score: ResMut<Score>,
) {
    for (mut velocity, ball_tf, ball_col) in &mut ball_query {
        let ball_pos = ball_tf.translation.truncate();
        let ball_half = Vec2::new(ball_col.half_width, ball_col.half_height);

        let mut reflected = false;

        for (brick_entity, brick_tf, brick_col, mut brick) in &mut brick_query {
            let brick_pos = brick_tf.translation.truncate();
            let brick_half = Vec2::new(brick_col.half_width, brick_col.half_height);

            if let Some(side) = aabb_collision(ball_pos, ball_half, brick_pos, brick_half) {
                brick.health = brick.health.saturating_sub(1);
                if brick.health == 0 {
                    score.value += brick.score_value;
                    commands.entity(brick_entity).despawn();
                }

                // Отражаем скорость только один раз за кадр
                if !reflected {
                    match side {
                        CollisionSide::Left | CollisionSide::Right => velocity.x = -velocity.x,
                        CollisionSide::Top | CollisionSide::Bottom => velocity.y = -velocity.y,
                    }
                    reflected = true;
                }
            }
        }
    }
}

/// Ball ↔ Paddle: отскок с угловым эффектом.
/// hit_factor зажат в [-0.8, 0.8] — мяч никогда не летит почти горизонтально.
/// velocity.y всегда строго положительный после отскока.
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
                        let speed =
                            (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();

                        // Нормированная точка удара [-1..1], зажатая до [-0.8..0.8]
                        // чтобы избежать почти горизонтального отскока
                        let hit_factor = ((ball_pos.x - paddle_pos.x) / paddle_half.x)
                            .clamp(-0.8, 0.8);

                        velocity.x = hit_factor * speed;
                        // Гарантируем минимальную вертикальную составляющую
                        let vy_sq = (speed * speed - velocity.x * velocity.x).max(0.0);
                        velocity.y = vy_sq.sqrt().max(speed * 0.4);

                        // Нормируем до исходной скорости
                        let new_speed =
                            (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
                        if new_speed > 0.0 {
                            velocity.x = velocity.x / new_speed * speed;
                            velocity.y = velocity.y / new_speed * speed;
                        }

                        // Выталкиваем мяч над ракеткой
                        let overlap = ball_half.y + paddle_half.y
                            - (ball_pos.y - paddle_pos.y).abs();
                        ball_tf.translation.y += overlap.max(0.0);
                    }
                    CollisionSide::Left | CollisionSide::Right => {
                        velocity.x = -velocity.x;
                    }
                }
            }
        }
    }
}
