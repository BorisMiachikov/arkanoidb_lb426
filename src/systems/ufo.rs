use rand::Rng;
use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::bomb::Bomb;
use crate::components::brick::Brick;
use crate::components::collider::Collider;
use crate::components::level_entity::LevelEntity;
use crate::components::paddle::Paddle;
use crate::components::ufo::Ufo;
use crate::components::velocity::Velocity;
use crate::resources::game_state::GameState;
use crate::resources::score::Lives;
use crate::setup::level::{HALF_W, WALL_THICKNESS};

const UFO_HALF_W: f32 = 30.0;
const BOMB_SIZE: f32 = 10.0;
const BOMB_SPEED: f32 = -220.0;

/// НЛО двигается горизонтально, отражается от стен, периодически сбрасывает бомбы
pub fn ufo_movement_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Ufo)>,
) {
    let dt = time.delta_secs().min(0.05);
    let bound = HALF_W - WALL_THICKNESS - UFO_HALF_W;

    for (_entity, mut transform, mut ufo) in &mut query {
        transform.translation.x += ufo.speed * ufo.direction * dt;

        // Отражение от боковых стен
        if transform.translation.x >= bound {
            transform.translation.x = bound;
            ufo.direction = -1.0;
        } else if transform.translation.x <= -bound {
            transform.translation.x = -bound;
            ufo.direction = 1.0;
        }

        // Сброс бомбы по таймеру
        ufo.bomb_timer.tick(time.delta());
        if ufo.bomb_timer.just_finished() {
            let x = transform.translation.x;
            let y = transform.translation.y;
            commands.spawn((
                LevelEntity,
                Bomb { damage: 1 },
                Collider::new(BOMB_SIZE, BOMB_SIZE),
                Velocity::new(0.0, BOMB_SPEED),
                Mesh2d(meshes.add(Circle::new(BOMB_SIZE / 2.0))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.3, 0.1))),
                Transform::from_xyz(x, y - 20.0, 0.9),
            ));
        }
    }
}

const UFO_W: f32 = 60.0;
const UFO_H: f32 = 24.0;
/// Зоны респавна НЛО: выше блоков (BRICKS_TOP=170, верх≈182) или ниже (низ≈18)
const UFO_RESPAWN_ABOVE_MIN: f32 = 200.0;
const UFO_RESPAWN_ABOVE_MAX: f32 = 270.0;
const UFO_RESPAWN_BELOW_MIN: f32 = -120.0;
const UFO_RESPAWN_BELOW_MAX: f32 =   0.0;

/// Ball ↔ UFO: мяч отражается, НЛО теряет здоровье (2 удара = уничтожение + респавн)
pub fn ball_ufo_collision_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ball_query: Query<(&mut Velocity, &mut Transform, &Collider), With<Ball>>,
    mut ufo_query: Query<(Entity, &Transform, &Collider, &mut Ufo), Without<Ball>>,
) {
    let mut rng = rand::thread_rng();

    for (mut velocity, mut ball_tf, ball_col) in &mut ball_query {
        let ball_half = Vec2::new(ball_col.half_width, ball_col.half_height);

        for (ufo_entity, ufo_tf, ufo_col, mut ufo) in &mut ufo_query {
            let ball_pos = ball_tf.translation.truncate();
            let ufo_pos = ufo_tf.translation.truncate();
            let ufo_half = Vec2::new(ufo_col.half_width, ufo_col.half_height);

            let dx = ball_pos.x - ufo_pos.x;
            let dy = ball_pos.y - ufo_pos.y;
            let overlap_x = ball_half.x + ufo_half.x - dx.abs();
            let overlap_y = ball_half.y + ufo_half.y - dy.abs();

            if overlap_x > 0.0 && overlap_y > 0.0 {
                // Отражение мяча
                if overlap_x < overlap_y {
                    velocity.x = -velocity.x;
                    ball_tf.translation.x += if dx > 0.0 { overlap_x } else { -overlap_x };
                } else {
                    velocity.y = -velocity.y;
                    ball_tf.translation.y += if dy > 0.0 { overlap_y } else { -overlap_y };
                }

                // Урон НЛО
                ufo.health = ufo.health.saturating_sub(1);
                if ufo.health == 0 {
                    let speed = ufo.speed;
                    let interval = ufo.bomb_timer.duration().as_secs_f32();
                    commands.entity(ufo_entity).despawn();

                    // Респавн случайно выше или ниже блоков
                    let new_y = if rng.gen_bool(0.5) {
                        rng.gen_range(UFO_RESPAWN_ABOVE_MIN..=UFO_RESPAWN_ABOVE_MAX)
                    } else {
                        rng.gen_range(UFO_RESPAWN_BELOW_MIN..=UFO_RESPAWN_BELOW_MAX)
                    };
                    commands.spawn((
                        LevelEntity,
                        Ufo::new(speed, interval),
                        Collider::new(UFO_W, UFO_H),
                        Mesh2d(meshes.add(Rectangle::new(UFO_W, UFO_H))),
                        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.2, 0.8))),
                        Transform::from_xyz(0.0, new_y, 1.0),
                    ));
                }
            }
        }
    }
}

/// Bomb ↔ Paddle: бомба попадает в ракетку — теряем жизнь
pub fn bomb_paddle_collision_system(
    mut commands: Commands,
    bomb_query: Query<(Entity, &Transform, &Collider), With<Bomb>>,
    paddle_query: Query<(&Transform, &Collider), With<Paddle>>,
    mut ball_query: Query<(Entity, &mut Transform, &mut Velocity), (With<Ball>, Without<Bomb>, Without<Paddle>)>,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((paddle_tf, paddle_col)) = paddle_query.get_single() else {
        return;
    };
    let paddle_pos = paddle_tf.translation.truncate();
    let paddle_half = Vec2::new(paddle_col.half_width, paddle_col.half_height);

    for (bomb_entity, bomb_tf, bomb_col) in &bomb_query {
        let bomb_pos = bomb_tf.translation.truncate();
        let bomb_half = Vec2::new(bomb_col.half_width, bomb_col.half_height);

        let hit = (bomb_pos.x - paddle_pos.x).abs() < bomb_half.x + paddle_half.x
            && (bomb_pos.y - paddle_pos.y).abs() < bomb_half.y + paddle_half.y;

        if hit {
            commands.entity(bomb_entity).despawn();
            lives.count = lives.count.saturating_sub(1);

            if lives.count == 0 {
                next_state.set(GameState::GameOver);
            } else {
                // Сброс мяча к ракетке
                for (ball_entity, mut ball_tf, mut ball_vel) in &mut ball_query {
                    ball_vel.x = 0.0;
                    ball_vel.y = 0.0;
                    ball_tf.translation.x = 0.0;
                    commands.entity(ball_entity).insert(BallStuck);
                }
            }
        }
    }
}

/// Bomb ↔ Brick: бомба попадает в блок — бомба исчезает
pub fn bomb_brick_collision_system(
    mut commands: Commands,
    bomb_query: Query<(Entity, &Transform, &Collider), With<Bomb>>,
    brick_query: Query<(&Transform, &Collider), With<Brick>>,
) {
    'bombs: for (bomb_entity, bomb_tf, bomb_col) in &bomb_query {
        let bomb_pos = bomb_tf.translation.truncate();
        let bomb_half = Vec2::new(bomb_col.half_width, bomb_col.half_height);

        for (brick_tf, brick_col) in &brick_query {
            let brick_pos = brick_tf.translation.truncate();
            let brick_half = Vec2::new(brick_col.half_width, brick_col.half_height);

            let hit = (bomb_pos.x - brick_pos.x).abs() < bomb_half.x + brick_half.x
                && (bomb_pos.y - brick_pos.y).abs() < bomb_half.y + brick_half.y;

            if hit {
                commands.entity(bomb_entity).despawn();
                continue 'bombs;
            }
        }
    }
}

/// UFO ↔ Brick: НЛО не проходит сквозь блоки, отражается от них горизонтально
pub fn ufo_brick_collision_system(
    brick_query: Query<(&Transform, &Collider), With<Brick>>,
    mut ufo_query: Query<(&mut Transform, &mut Ufo, &Collider), Without<Brick>>,
) {
    for (mut ufo_tf, mut ufo, ufo_col) in &mut ufo_query {
        let ufo_pos = ufo_tf.translation.truncate();
        let ufo_half = Vec2::new(ufo_col.half_width, ufo_col.half_height);

        for (brick_tf, brick_col) in &brick_query {
            let brick_pos = brick_tf.translation.truncate();
            let brick_half = Vec2::new(brick_col.half_width, brick_col.half_height);

            let dx = ufo_pos.x - brick_pos.x;
            let dy = ufo_pos.y - brick_pos.y;
            let overlap_x = ufo_half.x + brick_half.x - dx.abs();
            let overlap_y = ufo_half.y + brick_half.y - dy.abs();

            if overlap_x > 0.0 && overlap_y > 0.0 {
                if overlap_x < overlap_y {
                    // Горизонтальное столкновение — разворот
                    ufo.direction = -ufo.direction;
                    ufo_tf.translation.x += if dx > 0.0 { overlap_x } else { -overlap_x };
                } else {
                    // Вертикальное — выталкиваем
                    ufo_tf.translation.y += if dy > 0.0 { overlap_y } else { -overlap_y };
                }
            }
        }
    }
}

/// Бомбы вылетевшие за нижнюю границу — удаляем
pub fn cleanup_fallen_bombs_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Bomb>>,
) {
    for (entity, transform) in &query {
        if transform.translation.y < -350.0 {
            commands.entity(entity).despawn();
        }
    }
}
