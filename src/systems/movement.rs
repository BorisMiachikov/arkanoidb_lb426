use bevy::prelude::*;

use crate::components::velocity::Velocity;

/// Применение скорости к позиции (мяч, бонусы, бомбы)
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
