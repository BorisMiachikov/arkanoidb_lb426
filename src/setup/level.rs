use bevy::color::palettes::css;
use bevy::prelude::*;

use crate::components::ball::Ball;
use crate::components::collider::Collider;
use crate::components::paddle::Paddle;
use crate::components::velocity::Velocity;
use crate::components::wall::Wall;

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;
pub const HALF_W: f32 = WINDOW_WIDTH / 2.0;
pub const HALF_H: f32 = WINDOW_HEIGHT / 2.0;

pub const PADDLE_WIDTH: f32 = 120.0;
pub const PADDLE_HEIGHT: f32 = 18.0;
pub const PADDLE_Y: f32 = -HALF_H + 50.0;

pub const BALL_SIZE: f32 = 20.0;
pub const BALL_INITIAL_VX: f32 = 200.0;
pub const BALL_INITIAL_VY: f32 = 350.0;

pub const WALL_THICKNESS: f32 = 16.0;

/// Создаём ракетку, мяч и стены при входе в состояние Playing
pub fn spawn_level_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_paddle(&mut commands, &mut meshes, &mut materials);
    spawn_ball(&mut commands, &mut meshes, &mut materials);
    spawn_walls(&mut commands, &mut meshes, &mut materials);
}

fn spawn_paddle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Paddle::default(),
        Collider::new(PADDLE_WIDTH, PADDLE_HEIGHT),
        Mesh2d(meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT))),
        MeshMaterial2d(materials.add(Color::from(css::STEEL_BLUE))),
        Transform::from_xyz(0.0, PADDLE_Y, 1.0),
    ));
}

fn spawn_ball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Ball::default(),
        Collider::new(BALL_SIZE, BALL_SIZE),
        Velocity::new(BALL_INITIAL_VX, BALL_INITIAL_VY),
        Mesh2d(meshes.add(Circle::new(BALL_SIZE / 2.0))),
        MeshMaterial2d(materials.add(Color::from(css::WHITE))),
        Transform::from_xyz(0.0, PADDLE_Y + PADDLE_HEIGHT + BALL_SIZE, 1.0),
    ));
}

fn spawn_walls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let wall_color = Color::from(css::DARK_GRAY);

    // Левая стена
    commands.spawn((
        Wall,
        Collider::new(WALL_THICKNESS, WINDOW_HEIGHT),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, WINDOW_HEIGHT))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(-HALF_W + WALL_THICKNESS / 2.0, 0.0, 0.0),
    ));

    // Правая стена
    commands.spawn((
        Wall,
        Collider::new(WALL_THICKNESS, WINDOW_HEIGHT),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, WINDOW_HEIGHT))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(HALF_W - WALL_THICKNESS / 2.0, 0.0, 0.0),
    ));

    // Верхняя стена
    commands.spawn((
        Wall,
        Collider::new(WINDOW_WIDTH, WALL_THICKNESS),
        Mesh2d(meshes.add(Rectangle::new(WINDOW_WIDTH, WALL_THICKNESS))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(0.0, HALF_H - WALL_THICKNESS / 2.0, 0.0),
    ));

    // Нижней стены нет — мяч может упасть (TODO Этап 6: триггер потери мяча)
}
