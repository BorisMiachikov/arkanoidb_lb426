use bevy::color::palettes::css;
use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::brick::{Brick, BrickType};
use crate::components::collider::Collider;
use crate::components::level_entity::LevelEntity;
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

pub const BRICK_WIDTH: f32 = 72.0;
pub const BRICK_HEIGHT: f32 = 24.0;
const BRICK_COLS: usize = 10;
const BRICK_ROWS: usize = 5;
const BRICK_GAP: f32 = 4.0;
const BRICKS_TOP_Y: f32 = 170.0;

/// Создаём ракетку, мяч, стены и блоки при входе в состояние Playing
pub fn spawn_level_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_paddle(&mut commands, &mut meshes, &mut materials);
    spawn_ball(&mut commands, &mut meshes, &mut materials);
    spawn_walls(&mut commands, &mut meshes, &mut materials);
    spawn_bricks(&mut commands, &mut meshes, &mut materials);
}

/// Удаляем все сущности уровня при выходе из Playing
pub fn cleanup_level(mut commands: Commands, query: Query<Entity, With<LevelEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_paddle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        LevelEntity,
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
        LevelEntity,
        Ball::default(),
        BallStuck,
        Collider::new(BALL_SIZE, BALL_SIZE),
        Velocity::new(0.0, 0.0),
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
    let wall_color = Color::srgb(0.4, 0.4, 0.5);

    // Левая стена
    commands.spawn((
        LevelEntity,
        Wall,
        Collider::new(WALL_THICKNESS, WINDOW_HEIGHT),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, WINDOW_HEIGHT))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(-HALF_W + WALL_THICKNESS / 2.0, 0.0, 0.0),
    ));

    // Правая стена
    commands.spawn((
        LevelEntity,
        Wall,
        Collider::new(WALL_THICKNESS, WINDOW_HEIGHT),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, WINDOW_HEIGHT))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(HALF_W - WALL_THICKNESS / 2.0, 0.0, 0.0),
    ));

    // Верхняя стена
    commands.spawn((
        LevelEntity,
        Wall,
        Collider::new(WINDOW_WIDTH, WALL_THICKNESS),
        Mesh2d(meshes.add(Rectangle::new(WINDOW_WIDTH, WALL_THICKNESS))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(0.0, HALF_H - WALL_THICKNESS / 2.0, 0.0),
    ));
}

fn spawn_bricks(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let rows: [(Color, u32, u32); BRICK_ROWS] = [
        (Color::srgb(0.9, 0.2, 0.2), 2, 200), // красный — прочный
        (Color::srgb(0.9, 0.5, 0.1), 1, 150), // оранжевый
        (Color::srgb(0.9, 0.85, 0.1), 1, 100), // жёлтый
        (Color::srgb(0.2, 0.8, 0.2), 1, 75),  // зелёный
        (Color::srgb(0.2, 0.7, 0.9), 1, 50),  // голубой
    ];

    let total_w = BRICK_COLS as f32 * BRICK_WIDTH + (BRICK_COLS - 1) as f32 * BRICK_GAP;
    let start_x = -total_w / 2.0 + BRICK_WIDTH / 2.0;
    let step_x = BRICK_WIDTH + BRICK_GAP;
    let step_y = BRICK_HEIGHT + BRICK_GAP;

    for (row, (color, health, score_value)) in rows.iter().enumerate() {
        let brick_type = if *health > 1 { BrickType::Strong } else { BrickType::Normal };
        let y = BRICKS_TOP_Y - row as f32 * step_y;

        for col in 0..BRICK_COLS {
            let x = start_x + col as f32 * step_x;
            commands.spawn((
                LevelEntity,
                Brick { brick_type, health: *health, score_value: *score_value },
                Collider::new(BRICK_WIDTH, BRICK_HEIGHT),
                Mesh2d(meshes.add(Rectangle::new(BRICK_WIDTH, BRICK_HEIGHT))),
                MeshMaterial2d(materials.add(*color)),
                Transform::from_xyz(x, y, 0.5),
            ));
        }
    }
}
