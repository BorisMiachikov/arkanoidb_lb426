use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::bonus::Bonus;
use crate::components::bonus_effects::{BallGrowEffect, PaddleGrowEffect, StickyEffect};
use crate::components::collider::Collider;
use crate::components::paddle::Paddle;
use crate::setup::level::PADDLE_WIDTH;

const EFFECT_DURATION_SECS: f32 = 10.0;
const PADDLE_GROW_SCALE: f32 = 1.5;
const BALL_GROW_SCALE: f32 = 1.5;

/// AABB-проверка пересечения двух прямоугольников
fn overlaps(pos_a: Vec2, half_a: Vec2, pos_b: Vec2, half_b: Vec2) -> bool {
    (pos_a.x - pos_b.x).abs() < half_a.x + half_b.x
        && (pos_a.y - pos_b.y).abs() < half_a.y + half_b.y
}

/// Бонусы падают вниз — handled by apply_velocity_system (у них уже есть Velocity).

/// Подбор бонуса ракеткой: применяем эффект, удаляем бонус.
pub fn bonus_pickup_system(
    mut commands: Commands,
    bonus_query: Query<(Entity, &Transform, &Collider, &Bonus)>,
    paddle_query: Query<(Entity, &Transform, &Collider), With<Paddle>>,
    ball_query: Query<(Entity, &Collider), With<Ball>>,
) {
    let Ok((paddle_entity, paddle_tf, paddle_col)) = paddle_query.get_single() else {
        return;
    };
    let paddle_pos = paddle_tf.translation.truncate();
    let paddle_half = Vec2::new(paddle_col.half_width, paddle_col.half_height);

    for (bonus_entity, bonus_tf, bonus_col, bonus) in &bonus_query {
        let bonus_pos = bonus_tf.translation.truncate();
        let bonus_half = Vec2::new(bonus_col.half_width, bonus_col.half_height);

        if overlaps(paddle_pos, paddle_half, bonus_pos, bonus_half) {
            commands.entity(bonus_entity).despawn();

            match bonus.bonus_type {
                crate::components::bonus::BonusType::PaddleGrow => {
                    commands.entity(paddle_entity).insert(PaddleGrowEffect {
                        timer: Timer::from_seconds(EFFECT_DURATION_SECS, TimerMode::Once),
                        original_half_width: PADDLE_WIDTH / 2.0,
                    });
                }
                crate::components::bonus::BonusType::StickyPaddle => {
                    commands.entity(paddle_entity).insert(StickyEffect {
                        timer: Timer::from_seconds(EFFECT_DURATION_SECS, TimerMode::Once),
                    });
                }
                crate::components::bonus::BonusType::BallGrow => {
                    for (ball_entity, ball_col) in &ball_query {
                        commands.entity(ball_entity).insert(BallGrowEffect {
                            timer: Timer::from_seconds(EFFECT_DURATION_SECS, TimerMode::Once),
                            original_half_size: ball_col.half_width,
                        });
                    }
                }
                crate::components::bonus::BonusType::GunPaddle => {
                    // TODO Этап будущего
                }
            }
        }
    }
}

/// Применяем PaddleGrow при появлении компонента
pub fn apply_paddle_grow_system(
    mut query: Query<(&mut Transform, &mut Collider), Added<PaddleGrowEffect>>,
) {
    for (mut transform, mut collider) in &mut query {
        transform.scale.x = PADDLE_GROW_SCALE;
        collider.half_width *= PADDLE_GROW_SCALE;
    }
}

/// Применяем BallGrow при появлении компонента
pub fn apply_ball_grow_system(
    mut query: Query<(&mut Transform, &mut Collider), Added<BallGrowEffect>>,
) {
    for (mut transform, mut collider) in &mut query {
        transform.scale.x = BALL_GROW_SCALE;
        transform.scale.y = BALL_GROW_SCALE;
        collider.half_width *= BALL_GROW_SCALE;
        collider.half_height *= BALL_GROW_SCALE;
    }
}

/// Тикаем таймеры и отменяем эффекты по истечении
pub fn update_bonus_effects_system(
    mut commands: Commands,
    time: Res<Time>,
    mut paddle_grow_query: Query<
        (Entity, &mut Transform, &mut Collider, &mut PaddleGrowEffect),
        With<Paddle>,
    >,
    mut sticky_query: Query<(Entity, &mut StickyEffect), With<Paddle>>,
    mut ball_grow_query: Query<(Entity, &mut Transform, &mut Collider, &mut BallGrowEffect), With<Ball>>,
) {
    let dt = time.delta();

    // PaddleGrow
    for (entity, mut transform, mut collider, mut effect) in &mut paddle_grow_query {
        effect.timer.tick(dt);
        if effect.timer.just_finished() {
            transform.scale.x = 1.0;
            collider.half_width = effect.original_half_width;
            commands.entity(entity).remove::<PaddleGrowEffect>();
        }
    }

    // StickyPaddle
    for (entity, mut effect) in &mut sticky_query {
        effect.timer.tick(dt);
        if effect.timer.just_finished() {
            commands.entity(entity).remove::<StickyEffect>();
        }
    }

    // BallGrow
    for (entity, mut transform, mut collider, mut effect) in &mut ball_grow_query {
        effect.timer.tick(dt);
        if effect.timer.just_finished() {
            transform.scale.x = 1.0;
            transform.scale.y = 1.0;
            collider.half_width = effect.original_half_size;
            collider.half_height = effect.original_half_size;
            commands.entity(entity).remove::<BallGrowEffect>();
        }
    }
}

/// Мяч прилипает к ракетке если активен StickyEffect
/// Вызывается из ball_paddle_collision_system при обнаружении столкновения
pub fn is_sticky_active(sticky_query: &Query<(), (With<StickyEffect>, With<Paddle>)>) -> bool {
    !sticky_query.is_empty()
}
