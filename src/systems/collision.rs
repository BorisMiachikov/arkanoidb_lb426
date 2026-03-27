use rand::Rng;
use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::bonus::{Bonus, BonusType};
use crate::components::bonus_effects::{FireBallEffect, StickyEffect};
use crate::components::brick::Brick;
use crate::components::collider::Collider;
use crate::components::level_entity::LevelEntity;
use crate::components::paddle::Paddle;
use crate::components::velocity::Velocity;
use crate::components::wall::Wall;
use crate::events::SoundEvent;
use crate::resources::assets::GameAssets;
use crate::resources::score::Score;
use crate::setup::level::MAX_BALL_SPEED;
use crate::systems::particles::spawn_burst;

const BONUS_DROP_CHANCE: f64 = 0.30; // 30% шанс дропа бонуса
const BONUS_SIZE: f32 = 16.0;
const BONUS_FALL_SPEED: f32 = -180.0;

/// Сторона столкновения (с точки зрения мяча)
enum CollisionSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// AABB-столкновение двух прямоугольников.
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
pub fn ball_wall_collision_system(
    mut ball_query: Query<(&mut Velocity, &mut Transform, &Collider), With<Ball>>,
    wall_query: Query<(&Transform, &Collider), (With<Wall>, Without<Ball>)>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (mut velocity, mut ball_tf, ball_col) in &mut ball_query {
        let ball_half = Vec2::new(ball_col.half_width, ball_col.half_height);

        for (wall_tf, wall_col) in &wall_query {
            let ball_pos = ball_tf.translation.truncate();
            let wall_pos = wall_tf.translation.truncate();
            let wall_half = Vec2::new(wall_col.half_width, wall_col.half_height);

            if let Some(side) = aabb_collision(ball_pos, ball_half, wall_pos, wall_half) {
                sound_events.send(SoundEvent::BallHitWall);
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

/// Ball ↔ Brick: разрушение блоков, очки, дроп бонусов.
pub fn ball_brick_collision_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ball_query: Query<(&mut Velocity, &Transform, &Collider, Option<&FireBallEffect>), With<Ball>>,
    mut brick_query: Query<(Entity, &Transform, &Collider, &mut Brick, &mut Sprite)>,
    mut score: ResMut<Score>,
    mut sound_events: EventWriter<SoundEvent>,
    game_assets: Res<GameAssets>,
) {
    let mut rng = rand::thread_rng();

    for (mut velocity, ball_tf, ball_col, fire_effect) in &mut ball_query {
        let ball_pos = ball_tf.translation.truncate();
        let ball_half = Vec2::new(ball_col.half_width, ball_col.half_height);
        let is_fire = fire_effect.is_some();
        let mut reflected = false;

        for (brick_entity, brick_tf, brick_col, mut brick, mut brick_sprite) in &mut brick_query {
            let brick_pos = brick_tf.translation.truncate();
            let brick_half = Vec2::new(brick_col.half_width, brick_col.half_height);

            if let Some(side) = aabb_collision(ball_pos, ball_half, brick_pos, brick_half) {
                brick.health = brick.health.saturating_sub(1);

                if brick.health == 0 {
                    score.value += brick.score_value;
                    sound_events.send(SoundEvent::BrickBreak);

                    // Частицы взрыва цвета блока
                    let color = brick_sprite.color;
                    spawn_burst(&mut commands, &mut meshes, &mut materials, brick_pos, color, 8, 120.0, 0.4);

                    commands.entity(brick_entity).despawn();

                    // Случайный дроп бонуса
                    if rng.gen_bool(BONUS_DROP_CHANCE) {
                        let bonus_type = random_bonus_type(&mut rng);
                        let color = bonus_color(bonus_type);
                        commands.spawn((
                            LevelEntity,
                            Bonus { bonus_type },
                            Collider::new(BONUS_SIZE, BONUS_SIZE),
                            Velocity::new(0.0, BONUS_FALL_SPEED),
                            Mesh2d(meshes.add(Rectangle::new(BONUS_SIZE, BONUS_SIZE))),
                            MeshMaterial2d(materials.add(color)),
                            Transform::from_xyz(brick_pos.x, brick_pos.y, 0.8),
                        ));
                    }
                }

                if brick.health > 0 {
                    sound_events.send(SoundEvent::BallHitBrick);
                    // Strong brick: swap to damaged sprite after first hit
                    if brick.health == 1 {
                        brick_sprite.image = game_assets.sprite_brick_strong_hit.clone();
                    }
                }

                // FireBall: пробивает кирпичи насквозь — без отскока
                if !is_fire && !reflected {
                    match side {
                        CollisionSide::Left | CollisionSide::Right => velocity.x = -velocity.x,
                        CollisionSide::Top | CollisionSide::Bottom => velocity.y = -velocity.y,
                    }
                    reflected = true;

                    // Небольшое ускорение за каждый удар по кирпичу (классика Arkanoid)
                    let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
                    if speed > 0.0 {
                        let new_speed = (speed * 1.005).min(MAX_BALL_SPEED);
                        let scale = new_speed / speed;
                        velocity.x *= scale;
                        velocity.y *= scale;
                    }
                }
            }
        }
    }
}

/// Ball ↔ Paddle: отскок с угловым эффектом.
/// Если активен StickyEffect — мяч прилипает вместо отскока.
pub fn ball_paddle_collision_system(
    mut commands: Commands,
    mut ball_query: Query<(Entity, &mut Velocity, &mut Transform, &Collider), With<Ball>>,
    paddle_query: Query<(Entity, &Transform, &Collider, Option<&StickyEffect>), (With<Paddle>, Without<Ball>)>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (ball_entity, mut velocity, mut ball_tf, ball_col) in &mut ball_query {
        if velocity.y >= 0.0 {
            continue;
        }

        let ball_pos = ball_tf.translation.truncate();
        let ball_half = Vec2::new(ball_col.half_width, ball_col.half_height);

        for (_paddle_entity, paddle_tf, paddle_col, sticky) in &paddle_query {
            let paddle_pos = paddle_tf.translation.truncate();
            let paddle_half = Vec2::new(paddle_col.half_width, paddle_col.half_height);

            if let Some(side) = aabb_collision(ball_pos, ball_half, paddle_pos, paddle_half) {
                sound_events.send(SoundEvent::BallHitPaddle);
                match side {
                    CollisionSide::Top | CollisionSide::Bottom => {
                        let overlap = ball_half.y + paddle_half.y
                            - (ball_pos.y - paddle_pos.y).abs();
                        ball_tf.translation.y += overlap.max(0.0);

                        if sticky.is_some() {
                            // StickyEffect: мяч прилипает
                            velocity.x = 0.0;
                            velocity.y = 0.0;
                            commands.entity(ball_entity).insert(BallStuck);
                        } else {
                            let speed = (velocity.x * velocity.x
                                + velocity.y * velocity.y)
                                .sqrt();
                            let hit_factor =
                                ((ball_pos.x - paddle_pos.x) / paddle_half.x)
                                    .clamp(-0.8, 0.8);
                            velocity.x = hit_factor * speed;
                            let vy_sq =
                                (speed * speed - velocity.x * velocity.x).max(0.0);
                            velocity.y = vy_sq.sqrt().max(speed * 0.4);

                            let new_speed = (velocity.x * velocity.x
                                + velocity.y * velocity.y)
                                .sqrt();
                            if new_speed > 0.0 {
                                velocity.x = velocity.x / new_speed * speed;
                                velocity.y = velocity.y / new_speed * speed;
                            }
                        }
                    }
                    CollisionSide::Left | CollisionSide::Right => {
                        velocity.x = -velocity.x;
                    }
                }
            }
        }
    }
}

fn random_bonus_type(rng: &mut impl Rng) -> BonusType {
    match rng.gen_range(0..7) {
        0 => BonusType::PaddleGrow,
        1 => BonusType::StickyPaddle,
        2 => BonusType::BallGrow,
        3 => BonusType::GunPaddle,
        4 => BonusType::FireBall,
        5 => BonusType::MultiBall,
        _ => BonusType::ExtraLife,
    }
}

fn bonus_color(bonus_type: BonusType) -> Color {
    match bonus_type {
        BonusType::PaddleGrow   => Color::srgb(0.2, 0.9, 0.2), // зелёный
        BonusType::StickyPaddle => Color::srgb(0.9, 0.9, 0.1), // жёлтый
        BonusType::BallGrow     => Color::srgb(0.2, 0.8, 0.9), // голубой
        BonusType::GunPaddle    => Color::srgb(0.9, 0.4, 0.1), // оранжевый
        BonusType::FireBall     => Color::srgb(1.0, 0.3, 0.0), // красно-оранжевый
        BonusType::MultiBall    => Color::srgb(0.8, 0.3, 1.0), // фиолетовый
        BonusType::ExtraLife    => Color::srgb(1.0, 0.3, 0.5), // розовый
    }
}
