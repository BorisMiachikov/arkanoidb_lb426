use bevy::prelude::*;

/// Создаём 2D-камеру при старте
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
