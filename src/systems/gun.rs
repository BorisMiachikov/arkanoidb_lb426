use bevy::prelude::*;

use crate::components::bonus_effects::GunPaddleEffect;
use crate::components::brick::Brick;
use crate::components::bullet::Bullet;
use crate::components::collider::Collider;
use crate::components::level_entity::LevelEntity;
use crate::components::paddle::Paddle;
use crate::components::ufo::Ufo;
use crate::components::velocity::Velocity;
use crate::events::SoundEvent;
use crate::resources::score::Score;
use crate::setup::level::HALF_H;

const BULLET_W: f32 = 4.0;
const BULLET_H: f32 = 14.0;
const BULLET_SPEED: f32 = 520.0;

/// Стрельба из пулемёта: Z или LeftCtrl
pub fn fire_gun_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut paddle_query: Query<(&Transform, &mut GunPaddleEffect), With<Paddle>>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let fire = keys.pressed(KeyCode::ControlRight) || keys.pressed(KeyCode::ControlLeft);

    for (paddle_tf, mut effect) in &mut paddle_query {
        effect.fire_rate.tick(time.delta());

        if fire && effect.fire_rate.just_finished() {
            effect.fire_rate.reset();
            sound_events.send(SoundEvent::BulletFire);

            let x = paddle_tf.translation.x;
            let y = paddle_tf.translation.y + 14.0;

            commands.spawn((
                LevelEntity,
                Bullet,
                Collider::new(BULLET_W, BULLET_H),
                Velocity::new(0.0, BULLET_SPEED),
                Mesh2d(meshes.add(Rectangle::new(BULLET_W, BULLET_H))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.9, 0.1))),
                Transform::from_xyz(x, y, 2.0),
            ));
        }
    }
}

/// Bullet ↔ Brick: снаряд наносит 1 урон блоку и исчезает
pub fn bullet_brick_collision_system(
    mut commands: Commands,
    mut brick_query: Query<(Entity, &Transform, &Collider, &mut Brick)>,
    bullet_query: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    mut score: ResMut<Score>,
) {
    for (bullet_entity, bullet_tf, bullet_col) in &bullet_query {
        let bullet_pos = bullet_tf.translation.truncate();
        let bullet_half = Vec2::new(bullet_col.half_width, bullet_col.half_height);

        for (brick_entity, brick_tf, brick_col, mut brick) in &mut brick_query {
            let brick_pos = brick_tf.translation.truncate();
            let brick_half = Vec2::new(brick_col.half_width, brick_col.half_height);

            let hit = (bullet_pos.x - brick_pos.x).abs() < bullet_half.x + brick_half.x
                && (bullet_pos.y - brick_pos.y).abs() < bullet_half.y + brick_half.y;

            if hit {
                commands.entity(bullet_entity).despawn();
                brick.health = brick.health.saturating_sub(1);
                if brick.health == 0 {
                    score.value += brick.score_value;
                    commands.entity(brick_entity).despawn();
                }
                break;
            }
        }
    }
}

/// Bullet ↔ UFO: снаряд наносит 1 урон НЛО и исчезает
pub fn bullet_ufo_collision_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bullet_query: Query<(Entity, &Transform, &Collider), With<Bullet>>,
    mut ufo_query: Query<(Entity, &Transform, &Collider, &mut Ufo)>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    const UFO_W: f32 = 60.0;
    const UFO_H: f32 = 24.0;
    const ABOVE_MIN: f32 = 200.0;
    const ABOVE_MAX: f32 = 270.0;
    const BELOW_MIN: f32 = -120.0;
    const BELOW_MAX: f32 = 0.0;

    for (bullet_entity, bullet_tf, bullet_col) in &bullet_query {
        let bullet_pos = bullet_tf.translation.truncate();
        let bullet_half = Vec2::new(bullet_col.half_width, bullet_col.half_height);

        for (ufo_entity, ufo_tf, ufo_col, mut ufo) in &mut ufo_query {
            let ufo_pos = ufo_tf.translation.truncate();
            let ufo_half = Vec2::new(ufo_col.half_width, ufo_col.half_height);

            let hit = (bullet_pos.x - ufo_pos.x).abs() < bullet_half.x + ufo_half.x
                && (bullet_pos.y - ufo_pos.y).abs() < bullet_half.y + ufo_half.y;

            if hit {
                commands.entity(bullet_entity).despawn();
                ufo.health = ufo.health.saturating_sub(1);

                if ufo.health == 0 {
                    let speed = ufo.speed;
                    let interval = ufo.bomb_timer.duration().as_secs_f32();
                    commands.entity(ufo_entity).despawn();

                    let new_y = if rng.gen_bool(0.5) {
                        rng.gen_range(ABOVE_MIN..=ABOVE_MAX)
                    } else {
                        rng.gen_range(BELOW_MIN..=BELOW_MAX)
                    };
                    commands.spawn((
                        LevelEntity,
                        crate::components::ufo::Ufo::new(speed, interval),
                        Collider::new(UFO_W, UFO_H),
                        Mesh2d(meshes.add(Rectangle::new(UFO_W, UFO_H))),
                        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.2, 0.8))),
                        Transform::from_xyz(0.0, new_y, 1.0),
                    ));
                }
                break;
            }
        }
    }
}

/// Снаряды вылетевшие за верхнюю границу — удаляем
pub fn cleanup_bullets_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, tf) in &query {
        if tf.translation.y > HALF_H + 20.0 {
            commands.entity(entity).despawn();
        }
    }
}
