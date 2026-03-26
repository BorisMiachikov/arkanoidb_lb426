use bevy::prelude::*;

use crate::components::velocity::Velocity;

/// Применение скорости к позиции (мяч, бонусы, бомбы).
/// Работает в FixedUpdate (64 Hz), dt ≈ 0.016s — туннелирование исключено:
/// шаг мяча (≤12px) меньше минимального зазора коллизии (wall_half+ball_half=18px).
pub fn apply_velocity_system(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (velocity, mut transform) in &mut query {
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}
