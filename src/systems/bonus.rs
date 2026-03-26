use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::bonus::Bonus;
use crate::components::bonus_effects::{
    BallGrowEffect, FireBallEffect, GunPaddleEffect, PaddleGrowEffect, StickyEffect,
};
use crate::components::collider::Collider;
use crate::components::level_entity::LevelEntity;
use crate::components::paddle::Paddle;
use crate::components::velocity::Velocity;
use crate::events::SoundEvent;
use crate::setup::level::{BALL_SIZE, PADDLE_WIDTH};

const EFFECT_DURATION_SECS: f32 = 10.0;
const FIREBALL_DURATION_SECS: f32 = 8.0;
const PADDLE_GROW_SCALE: f32 = 1.5;
const BALL_GROW_SCALE: f32 = 1.5;

/// Поворот вектора скорости на angle (радианы)
fn rotate_vel(v: Vec2, angle: f32) -> Vec2 {
    Vec2::new(
        v.x * angle.cos() - v.y * angle.sin(),
        v.x * angle.sin() + v.y * angle.cos(),
    )
}

/// AABB-проверка пересечения двух прямоугольников
fn overlaps(pos_a: Vec2, half_a: Vec2, pos_b: Vec2, half_b: Vec2) -> bool {
    (pos_a.x - pos_b.x).abs() < half_a.x + half_b.x
        && (pos_a.y - pos_b.y).abs() < half_a.y + half_b.y
}

/// Бонусы падают вниз — handled by apply_velocity_system (у них уже есть Velocity).

/// Подбор бонуса ракеткой: применяем эффект, удаляем бонус.
pub fn bonus_pickup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bonus_query: Query<(Entity, &Transform, &Collider, &Bonus)>,
    paddle_query: Query<(Entity, &Transform, &Collider), With<Paddle>>,
    ball_query: Query<(Entity, &Transform, &Velocity, &Collider, Option<&BallStuck>), With<Ball>>,
    mut sound_events: EventWriter<SoundEvent>,
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
            sound_events.send(SoundEvent::BonusPickup);

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
                    for (ball_entity, _, _, ball_col, _) in &ball_query {
                        commands.entity(ball_entity).insert(BallGrowEffect {
                            timer: Timer::from_seconds(EFFECT_DURATION_SECS, TimerMode::Once),
                            original_half_size: ball_col.half_width,
                        });
                    }
                }
                crate::components::bonus::BonusType::GunPaddle => {
                    commands.entity(paddle_entity).insert(GunPaddleEffect {
                        timer: Timer::from_seconds(15.0, TimerMode::Once),
                        fire_rate: Timer::from_seconds(0.18, TimerMode::Repeating),
                    });
                }
                crate::components::bonus::BonusType::FireBall => {
                    for (ball_entity, _, _, _, _) in &ball_query {
                        commands.entity(ball_entity).insert(FireBallEffect {
                            timer: Timer::from_seconds(FIREBALL_DURATION_SECS, TimerMode::Once),
                        });
                    }
                }
                crate::components::bonus::BonusType::MultiBall => {
                    // Спавним 2 дополнительных мяча для каждого незастрявшего
                    let mut extras: Vec<(Vec3, Vec2)> = Vec::new();
                    for (_, ball_tf, ball_vel, _, stuck) in &ball_query {
                        if stuck.is_some() {
                            continue;
                        }
                        let v = Vec2::new(ball_vel.x, ball_vel.y);
                        extras.push((ball_tf.translation, rotate_vel(v, 25.0_f32.to_radians())));
                        extras.push((ball_tf.translation, rotate_vel(v, -25.0_f32.to_radians())));
                    }
                    for (pos, vel) in extras {
                        spawn_extra_ball(&mut commands, &mut meshes, &mut materials, pos, vel);
                    }
                }
            }
        }
    }
}

fn spawn_extra_ball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec3,
    vel: Vec2,
) {
    commands.spawn((
        LevelEntity,
        Ball::default(),
        Collider::new(BALL_SIZE, BALL_SIZE),
        Velocity::new(vel.x, vel.y),
        Mesh2d(meshes.add(Circle::new(BALL_SIZE / 2.0))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_xyz(pos.x, pos.y, 1.0),
    ));
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

/// Применяем FireBallEffect: мяч становится оранжевым
pub fn apply_fireball_effect_system(
    query: Query<&MeshMaterial2d<ColorMaterial>, Added<FireBallEffect>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for mat_handle in &query {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = Color::srgb(1.0, 0.45, 0.0); // оранжевый
        }
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut paddle_grow_query: Query<
        (Entity, &mut Transform, &mut Collider, &mut PaddleGrowEffect),
        (With<Paddle>, Without<Ball>),
    >,
    mut sticky_query: Query<(Entity, &mut StickyEffect), (With<Paddle>, Without<Ball>)>,
    mut ball_grow_query: Query<
        (Entity, &mut Transform, &mut Collider, &mut BallGrowEffect),
        (With<Ball>, Without<Paddle>),
    >,
    mut gun_query: Query<(Entity, &mut GunPaddleEffect), With<Paddle>>,
    mut fireball_query: Query<
        (Entity, &mut FireBallEffect, &MeshMaterial2d<ColorMaterial>),
        With<Ball>,
    >,
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

    // GunPaddle
    for (entity, mut effect) in &mut gun_query {
        effect.timer.tick(dt);
        if effect.timer.just_finished() {
            commands.entity(entity).remove::<GunPaddleEffect>();
        }
    }

    // FireBall
    for (entity, mut effect, mat_handle) in &mut fireball_query {
        effect.timer.tick(dt);
        if effect.timer.just_finished() {
            // Восстанавливаем белый цвет мяча
            if let Some(mat) = materials.get_mut(&mat_handle.0) {
                mat.color = Color::WHITE;
            }
            commands.entity(entity).remove::<FireBallEffect>();
        }
    }
}

/// Мяч прилипает к ракетке если активен StickyEffect
/// Вызывается из ball_paddle_collision_system при обнаружении столкновения
pub fn is_sticky_active(sticky_query: &Query<(), (With<StickyEffect>, With<Paddle>)>) -> bool {
    !sticky_query.is_empty()
}
