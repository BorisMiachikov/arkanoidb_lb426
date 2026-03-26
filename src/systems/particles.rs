use rand::Rng;
use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::bonus_effects::FireBallEffect;
use crate::components::level_entity::LevelEntity;
use crate::components::particle::Particle;
use crate::components::velocity::Velocity;

// ─── Ресурс таймера следа ────────────────────────────────────────────────────

/// Таймер для спавна следа мяча (срабатывает ~25 раз в секунду)
#[derive(Resource)]
pub struct BallTrailTimer(pub Timer);

impl Default for BallTrailTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.04, TimerMode::Repeating))
    }
}

// ─── Хелпер: взрыв частиц ────────────────────────────────────────────────────

/// Спавн N частиц в случайных направлениях — взрыв/разлёт осколков
pub fn spawn_burst(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    color: Color,
    count: u32,
    speed: f32,
    lifetime_secs: f32,
) {
    let mut rng = rand::thread_rng();
    let mesh = meshes.add(Rectangle::new(4.0, 4.0));

    for _ in 0..count {
        let angle = rng.gen_range(0.0_f32..std::f32::consts::TAU);
        let s = rng.gen_range(0.5..1.5) * speed;
        let lt = rng.gen_range(lifetime_secs * 0.5..lifetime_secs);

        commands.spawn((
            LevelEntity,
            Particle {
                lifetime: Timer::from_seconds(lt, TimerMode::Once),
            },
            Velocity::new(angle.cos() * s, angle.sin() * s),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(position.x, position.y, 2.0),
        ));
    }
}

// ─── Системы ─────────────────────────────────────────────────────────────────

/// Тикает таймеры частиц; уменьшает и затухает по мере жизни, удаляет по истечении
pub fn update_particles_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Particle, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta();
    for (entity, mut particle, mut transform, mat_handle) in &mut query {
        particle.lifetime.tick(dt);

        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let frac = particle.lifetime.fraction_remaining();

        // Масштаб уменьшается по мере затухания
        transform.scale = Vec3::splat(frac.max(0.05));

        // Альфа-канал затухает
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let c = mat.color.to_linear();
            mat.color = Color::linear_rgba(c.red, c.green, c.blue, frac);
        }
    }
}

/// Спавн следа позади движущихся мячей (не прилипших)
pub fn ball_trail_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut trail_timer: ResMut<BallTrailTimer>,
    ball_query: Query<(&Transform, Option<&FireBallEffect>), (With<Ball>, Without<BallStuck>)>,
) {
    trail_timer.0.tick(time.delta());
    if !trail_timer.0.just_finished() {
        return;
    }

    let mesh = meshes.add(Circle::new(5.0));
    for (transform, fire_effect) in &ball_query {
        let color = if fire_effect.is_some() {
            Color::linear_rgba(1.0, 0.4, 0.0, 0.5) // оранжевый след для FireBall
        } else {
            Color::linear_rgba(0.6, 0.8, 1.0, 0.35) // голубоватый след
        };

        commands.spawn((
            LevelEntity,
            Particle {
                lifetime: Timer::from_seconds(0.18, TimerMode::Once),
            },
            Velocity::new(0.0, 0.0),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                transform.translation.x,
                transform.translation.y,
                0.3,
            ),
        ));
    }
}
