use bevy::prelude::*;

use crate::resources::editor::{editor_cell_color, EditorData, EDITOR_COLS};
use crate::resources::game_state::GameState;

// ─── Маркеры ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct EditorCell {
    pub row: usize,
    pub col: usize,
}

/// Маркер для всего UI редактора (для очистки при выходе)
#[derive(Component)]
pub struct EditorEntity;

// ─── Вспомогательные ────────────────────────────────────────────────────────

fn brush_label(brush: u8) -> &'static str {
    match brush {
        1 => "NORMAL (blue)",
        2 => "STRONG (red)",
        _ => "ERASE (empty)",
    }
}

// ─── Setup / Cleanup ────────────────────────────────────────────────────────

/// Создаёт редактор при входе в LevelEditor
pub fn setup_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut editor: ResMut<EditorData>,
) {
    editor.needs_redraw = false;
    editor.last_painted = None;

    spawn_editor_cells(&mut commands, &mut meshes, &mut materials, &mut editor);
    spawn_editor_ui(&mut commands, &editor);
}

/// Удаляет все сущности редактора при выходе
pub fn cleanup_editor(
    mut commands: Commands,
    query: Query<Entity, With<EditorEntity>>,
    mut editor: ResMut<EditorData>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    editor.cell_materials.clear();
}

fn spawn_editor_cells(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    editor: &mut EditorData,
) {
    use crate::setup::level::{BRICK_HEIGHT, BRICK_WIDTH};

    editor.cell_materials.clear();
    let total_cells = editor.rows * EDITOR_COLS;
    editor.cell_materials.reserve(total_cells);

    let mesh = meshes.add(Rectangle::new(BRICK_WIDTH, BRICK_HEIGHT));

    for row in 0..editor.rows {
        for col in 0..EDITOR_COLS {
            let color = editor_cell_color(editor.grid[row][col]);
            let mat = materials.add(color);
            editor.cell_materials.push(mat.clone());

            let pos = EditorData::cell_world_pos(row, col);
            commands.spawn((
                EditorEntity,
                EditorCell { row, col },
                Mesh2d(mesh.clone()),
                MeshMaterial2d(mat),
                Transform::from_xyz(pos.x, pos.y, 0.5),
            ));
        }
    }
}

fn spawn_editor_ui(commands: &mut Commands, editor: &EditorData) {
    commands
        .spawn((
            EditorEntity,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Верхняя строка: заголовок
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("LEVEL EDITOR"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::srgb(0.4, 0.9, 1.0)),
                ));
                row.spawn((
                    Text::new(format!("Ряды: {}  (+/-)", editor.rows)),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    EditorRowsText,
                ));
            });

            // Строка кисти
            root.spawn(Node { justify_content: JustifyContent::Center, ..default() })
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!("Кисть [0/1/2]: {}", brush_label(editor.brush))),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                        EditorBrushText,
                    ));
                });

            // Нижняя строка: команды
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("[S] Сохранить   [L] Загрузить   [P] Играть"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.5, 1.0, 0.5)),
                ));
                row.spawn((
                    Text::new("[ESC] В меню"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            });
        });
}

// ─── Компоненты-маркеры UI ──────────────────────────────────────────────────

#[derive(Component)]
pub(crate) struct EditorBrushText;

#[derive(Component)]
pub(crate) struct EditorRowsText;

// ─── Ввод мышью ─────────────────────────────────────────────────────────────

/// Обработка кликов / перетаскивания по ячейкам
pub fn editor_mouse_system(
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut editor: ResMut<EditorData>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let pressing_left = mouse.pressed(MouseButton::Left);
    let pressing_right = mouse.pressed(MouseButton::Right);
    if !pressing_left && !pressing_right {
        editor.last_painted = None;
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_screen) = window.cursor_position() else { return };
    let Ok((camera, cam_tf)) = camera.get_single() else { return };
    let Ok(cursor_world) = camera.viewport_to_world_2d(cam_tf, cursor_screen) else { return };

    let Some((row, col)) = EditorData::world_to_cell(cursor_world, editor.rows) else {
        return;
    };

    // Не красить одну ячейку дважды подряд (drag)
    if editor.last_painted == Some((row, col)) {
        return;
    }
    editor.last_painted = Some((row, col));

    let new_val = if pressing_right {
        0
    } else {
        editor.brush
    };

    editor.grid[row][col] = new_val;

    // Обновить цвет материала ячейки
    let idx = row * EDITOR_COLS + col;
    if let Some(handle) = editor.cell_materials.get(idx) {
        if let Some(mat) = materials.get_mut(handle) {
            mat.color = editor_cell_color(new_val);
        }
    }
}

// ─── Ввод клавиатуры ─────────────────────────────────────────────────────────

/// Обработка клавиш редактора
pub fn editor_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut editor: ResMut<EditorData>,
    mut brush_text: Query<&mut Text, (With<EditorBrushText>, Without<EditorRowsText>)>,
    mut rows_text: Query<&mut Text, (With<EditorRowsText>, Without<EditorBrushText>)>,
) {
    // Кисть: клавиши 0, 1, 2
    let new_brush = if keys.just_pressed(KeyCode::Digit0) {
        Some(0u8)
    } else if keys.just_pressed(KeyCode::Digit1) {
        Some(1u8)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(2u8)
    } else {
        None
    };
    if let Some(b) = new_brush {
        editor.brush = b;
        if let Ok(mut t) = brush_text.get_single_mut() {
            **t = format!("Кисть [0/1/2]: {}", brush_label(b));
        }
    }

    // Ряды: + / -
    let add = keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd);
    let sub = keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract);
    if add {
        editor.add_row();
    } else if sub {
        editor.remove_row();
    }
    if add || sub {
        if let Ok(mut t) = rows_text.get_single_mut() {
            **t = format!("Ряды: {}  (+/-)", editor.rows);
        }
    }

    // S — сохранить
    if keys.just_pressed(KeyCode::KeyS) {
        editor.save();
    }

    // L — загрузить
    if keys.just_pressed(KeyCode::KeyL) {
        editor.load();
        // needs_redraw уже выставлен внутри load()
    }

    // P — играть кастомный уровень
    if keys.just_pressed(KeyCode::KeyP) {
        editor.active = true;
        next_state.set(GameState::Playing);
    }

    // Escape — в меню
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}

// ─── Перерисовка при изменении рядов / загрузке ─────────────────────────────

/// Пересоздаёт ячейки, если was выставлен needs_redraw
pub fn editor_redraw_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut editor: ResMut<EditorData>,
    cells_query: Query<Entity, With<EditorCell>>,
    mut rows_text: Query<&mut Text, With<EditorRowsText>>,
    mut brush_text: Query<&mut Text, (With<EditorBrushText>, Without<EditorRowsText>)>,
) {
    if !editor.needs_redraw {
        return;
    }
    editor.needs_redraw = false;

    // Удалить старые ячейки
    for entity in &cells_query {
        commands.entity(entity).despawn();
    }
    editor.cell_materials.clear();

    // Создать новые
    spawn_editor_cells(&mut commands, &mut meshes, &mut materials, &mut editor);

    // Обновить UI
    if let Ok(mut t) = rows_text.get_single_mut() {
        **t = format!("Ряды: {}  (+/-)", editor.rows);
    }
    if let Ok(mut t) = brush_text.get_single_mut() {
        **t = format!("Кисть [0/1/2]: {}", brush_label(editor.brush));
    }
}
