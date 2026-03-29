use bevy::prelude::*;
use bevy::camera::ScalingMode;

/// Создаём 2D-камеру при старте.
/// ScalingMode::AutoMin сохраняет виртуальное пространство 800×600,
/// масштабируя его под размер окна с letterbox/pillarbox.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 800.0,
                min_height: 600.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
