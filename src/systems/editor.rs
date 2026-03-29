use bevy::prelude::*;

use crate::resources::editor::{editor_cell_color, EditorData, EDITOR_COLS};
use crate::resources::game_state::GameState;
use crate::setup::level::{encode_cell, BRICK_COLOR_NAMES};

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

fn brush_label(brush: u8) -> String {
    if brush == 0 {
        return "ERASE".to_string();
    }
    let (type_name, color_idx) = if brush > 6 {
        ("STRONG", (brush - 7) as usize)
    } else {
        ("NORMAL", (brush - 1) as usize)
    };
    format!("{} + {}", type_name, BRICK_COLOR_NAMES[color_idx])
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
        commands.entity(entity).despawn();
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
            // Верхняя строка: заголовок | текущий уровень | ряды
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("LEVEL EDITOR"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::srgb(0.4, 0.9, 1.0)),
                ));
                row.spawn((
                    Text::new(format!("< {} >  [</> nav]", editor.level_label())),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    EditorLevelText,
                ));
                row.spawn((
                    Text::new(format!("Rows: {}  (+/-)", editor.rows)),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    EditorRowsText,
                ));
            });

            // Строка кисти
            root.spawn(Node { justify_content: JustifyContent::Center, ..default() })
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!("[1-6] Color  [T] Type  [0] Erase  |  {}", brush_label(editor.brush))),
                        TextFont { font_size: 15.0, ..default() },
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
                    Text::new("[N] New   [S] Save   [L] Load   [P] Play"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.5, 1.0, 0.5)),
                ));
                row.spawn((
                    Text::new("[ESC] Menu"),
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

#[derive(Component)]
pub(crate) struct EditorLevelText;

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

    let Ok(window) = windows.single() else { return };
    let Some(cursor_screen) = window.cursor_position() else { return };
    let Ok((camera, cam_tf)) = camera.single() else { return };
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
    mut brush_text: Query<
        &mut Text,
        (With<EditorBrushText>, Without<EditorRowsText>, Without<EditorLevelText>),
    >,
    mut rows_text: Query<
        &mut Text,
        (With<EditorRowsText>, Without<EditorBrushText>, Without<EditorLevelText>),
    >,
    mut level_text: Query<
        &mut Text,
        (With<EditorLevelText>, Without<EditorBrushText>, Without<EditorRowsText>),
    >,
) {
    // Навигация по уровням: стрелки влево / вправо
    let prev = keys.just_pressed(KeyCode::ArrowLeft);
    let next = keys.just_pressed(KeyCode::ArrowRight);
    if prev || next {
        editor.switch_level(if next { 1 } else { -1 });
        // Обновим текст уровня сразу (ячейки перерисует redraw_system)
        if let Ok(mut t) = level_text.single_mut() {
            **t = format!("< {} >  [</> nav]", editor.level_label());
        }
        if let Ok(mut t) = rows_text.single_mut() {
            **t = format!("Rows: {}  (+/-)", editor.rows);
        }
        return; // не обрабатывать остальные клавиши в том же кадре
    }

    // 0 — стёрка
    if keys.just_pressed(KeyCode::Digit0) {
        editor.brush = 0;
        if let Ok(mut t) = brush_text.single_mut() {
            **t = format!("[1-6] Color  [T] Type  [0] Erase  |  {}", brush_label(0));
        }
    }

    // 1–6 — выбор цвета (тип сохраняется)
    let color_keys = [
        (KeyCode::Digit1, 0usize),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
    ];
    for (key, ci) in color_keys {
        if keys.just_pressed(key) {
            let is_strong = editor.brush > 6;
            editor.brush = encode_cell(if is_strong { 2 } else { 1 }, ci);
            if let Ok(mut t) = brush_text.single_mut() {
                **t = format!("[1-6] Color  [T] Type  [0] Erase  |  {}", brush_label(editor.brush));
            }
            break;
        }
    }

    // T — переключить тип Normal ↔ Strong
    if keys.just_pressed(KeyCode::KeyT) && editor.brush > 0 {
        let is_strong = editor.brush > 6;
        let ci = if is_strong { (editor.brush - 7) as usize } else { (editor.brush - 1) as usize };
        editor.brush = encode_cell(if is_strong { 1 } else { 2 }, ci);
        if let Ok(mut t) = brush_text.single_mut() {
            **t = format!("[1-6] Color  [T] Type  [0] Erase  |  {}", brush_label(editor.brush));
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
        if let Ok(mut t) = rows_text.single_mut() {
            **t = format!("Rows: {}  (+/-)", editor.rows);
        }
    }

    // N — создать новый уровень после последнего
    if keys.just_pressed(KeyCode::KeyN) {
        editor.new_level();
        if let Ok(mut t) = level_text.single_mut() {
            **t = format!("< {} >  [</> nav]", editor.level_label());
        }
        if let Ok(mut t) = rows_text.single_mut() {
            **t = format!("Rows: {}  (+/-)", editor.rows);
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

/// Пересоздаёт ячейки, если был выставлен needs_redraw
pub fn editor_redraw_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut editor: ResMut<EditorData>,
    cells_query: Query<Entity, With<EditorCell>>,
    mut rows_text: Query<
        &mut Text,
        (With<EditorRowsText>, Without<EditorBrushText>, Without<EditorLevelText>),
    >,
    mut brush_text: Query<
        &mut Text,
        (With<EditorBrushText>, Without<EditorRowsText>, Without<EditorLevelText>),
    >,
    mut level_text: Query<
        &mut Text,
        (With<EditorLevelText>, Without<EditorBrushText>, Without<EditorRowsText>),
    >,
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
    if let Ok(mut t) = rows_text.single_mut() {
        **t = format!("Rows: {}  (+/-)", editor.rows);
    }
    if let Ok(mut t) = brush_text.single_mut() {
        **t = format!("[1-6] Color  [T] Type  [0] Erase  |  {}", brush_label(editor.brush));
    }
    if let Ok(mut t) = level_text.single_mut() {
        **t = format!("< {} >  [</> nav]", editor.level_label());
    }
}
