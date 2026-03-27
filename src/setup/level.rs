use bevy::color::palettes::css;
use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::brick::{Brick, BrickType};
use crate::components::ufo::Ufo;
use crate::components::collider::Collider;
use crate::components::level_entity::LevelEntity;
use crate::components::paddle::Paddle;
use crate::components::velocity::Velocity;
use crate::components::wall::Wall;
use crate::resources::assets::GameAssets;
use crate::resources::editor::EditorData;
use crate::resources::level_data::{LevelConfig, LEVELS};
use crate::resources::score::{BallSpeedMultiplier, CurrentLevel};

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
pub const MAX_BALL_SPEED: f32 = 750.0;

pub const WALL_THICKNESS: f32 = 16.0;

pub const BRICK_WIDTH: f32 = 72.0;
pub const BRICK_HEIGHT: f32 = 24.0;
pub const BRICK_GAP: f32 = 4.0;
pub const BRICKS_TOP_Y: f32 = 170.0;

/// Создаём сущности уровня.
/// Если EditorData.active — используем кастомную сетку из редактора.
/// Иначе — берём из LEVELS[CurrentLevel].
pub fn spawn_level_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    current_level: Res<CurrentLevel>,
    mut speed_multiplier: ResMut<BallSpeedMultiplier>,
    editor_data: Res<EditorData>,
    game_assets: Res<GameAssets>,
) {
    spawn_paddle(&mut commands, &game_assets);
    spawn_ball(&mut commands, &mut meshes, &mut materials);
    spawn_walls(&mut commands, &mut meshes, &mut materials);

    if editor_data.active {
        // Кастомный уровень из редактора — без НЛО, базовая скорость
        speed_multiplier.0 = 1.0;
        let grid_refs: Vec<&[u8]> = editor_data.grid.iter().map(|r| r.as_slice()).collect();
        spawn_bricks(&mut commands, &game_assets, &grid_refs);
    } else {
        let level_idx = (current_level.number as usize).min(LEVELS.len() - 1);
        let config = &LEVELS[level_idx];
        speed_multiplier.0 = config.ball_speed_multiplier;
        spawn_bricks(&mut commands, &game_assets, config.grid);
        spawn_ufos(&mut commands, &mut meshes, &mut materials, config);
    }
}

/// Удаляем все сущности уровня при выходе из Playing
pub fn cleanup_level(mut commands: Commands, query: Query<Entity, With<LevelEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_paddle(
    commands: &mut Commands,
    game_assets: &Res<GameAssets>,
) {
    commands.spawn((
        LevelEntity,
        Paddle::default(),
        Collider::new(PADDLE_WIDTH, PADDLE_HEIGHT),
        Sprite {
            image: game_assets.sprite_paddle.clone(),
            custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            ..default()
        },
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

    commands.spawn((
        LevelEntity, Wall,
        Collider::new(WALL_THICKNESS, WINDOW_HEIGHT),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, WINDOW_HEIGHT))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(-HALF_W + WALL_THICKNESS / 2.0, 0.0, 0.0),
    ));
    commands.spawn((
        LevelEntity, Wall,
        Collider::new(WALL_THICKNESS, WINDOW_HEIGHT),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, WINDOW_HEIGHT))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(HALF_W - WALL_THICKNESS / 2.0, 0.0, 0.0),
    ));
    commands.spawn((
        LevelEntity, Wall,
        Collider::new(WINDOW_WIDTH, WALL_THICKNESS),
        Mesh2d(meshes.add(Rectangle::new(WINDOW_WIDTH, WALL_THICKNESS))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_xyz(0.0, HALF_H - WALL_THICKNESS / 2.0, 0.0),
    ));
}

fn spawn_ufos(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    config: &LevelConfig,
) {
    const UFO_W: f32 = 60.0;
    const UFO_H: f32 = 24.0;

    for &(x, y) in config.ufos {
        commands.spawn((
            LevelEntity,
            Ufo::new(config.ufo_speed, config.ufo_bomb_interval),
            Collider::new(UFO_W, UFO_H),
            Mesh2d(meshes.add(Rectangle::new(UFO_W, UFO_H))),
            MeshMaterial2d(materials.add(Color::srgb(0.8, 0.2, 0.8))),
            Transform::from_xyz(x, y, 1.0),
        ));
    }
}

fn spawn_bricks(
    commands: &mut Commands,
    game_assets: &Res<GameAssets>,
    grid: &[&[u8]],
) {
    // Цвета рядов для обычных блоков (тип 1) — используются как тинт спрайта
    let row_colors = [
        Color::srgb(0.2, 0.7, 0.9), // голубой
        Color::srgb(0.2, 0.8, 0.2), // зелёный
        Color::srgb(0.9, 0.85, 0.1), // жёлтый
        Color::srgb(0.9, 0.5, 0.1), // оранжевый
        Color::srgb(0.7, 0.3, 0.9), // фиолетовый
        Color::srgb(0.2, 0.7, 0.9), // повтор
    ];

    let cols = grid.iter().map(|r| r.len()).max().unwrap_or(0);
    let total_w = cols as f32 * BRICK_WIDTH + (cols.saturating_sub(1)) as f32 * BRICK_GAP;
    let start_x = -total_w / 2.0 + BRICK_WIDTH / 2.0;
    let step_x = BRICK_WIDTH + BRICK_GAP;
    let step_y = BRICK_HEIGHT + BRICK_GAP;

    for (row, row_data) in grid.iter().enumerate() {
        let y = BRICKS_TOP_Y - row as f32 * step_y;

        for (col, &cell) in row_data.iter().enumerate() {
            if cell == 0 {
                continue;
            }
            let x = start_x + col as f32 * step_x;
            let (brick_type, health, score_value, image, color) = match cell {
                2 => (
                    BrickType::Strong,
                    2u32,
                    200u32,
                    game_assets.sprite_brick_strong.clone(),
                    Color::WHITE,
                ),
                _ => (
                    BrickType::Normal,
                    1,
                    100,
                    game_assets.sprite_brick_normal.clone(),
                    row_colors[row % row_colors.len()],
                ),
            };

            commands.spawn((
                LevelEntity,
                Brick { brick_type, health, score_value },
                Collider::new(BRICK_WIDTH, BRICK_HEIGHT),
                Sprite {
                    image,
                    custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                    color,
                    ..default()
                },
                Transform::from_xyz(x, y, 0.5),
            ));
        }
    }
}
