use bevy::prelude::*;

use crate::components::velocity::Velocity;

/// Применение скорости к позиции (мяч, бонусы, бомбы).
/// delta_secs ограничен 0.05s, чтобы избежать туннелирования
/// сквозь стены в первых кадрах при компиляции шейдеров.
pub fn apply_velocity_system(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (velocity, mut transform) in &mut query {
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}
