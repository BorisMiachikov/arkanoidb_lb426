use bevy::prelude::*;

use crate::setup::level::{BRICK_GAP, BRICK_HEIGHT, BRICK_WIDTH, BRICKS_TOP_Y};

pub const EDITOR_COLS: usize = 10;
pub const EDITOR_MIN_ROWS: usize = 1;
pub const EDITOR_MAX_ROWS: usize = 10;
pub const EDITOR_FILE: &str = "custom_level.lvl";

/// Тип ячейки при наведении/перетаскивании (чтобы не красить одну ячейку дважды)
#[derive(Resource)]
pub struct EditorData {
    /// grid[row][col] — 0=пусто, 1=обычный, 2=прочный
    pub grid: Vec<Vec<u8>>,
    pub rows: usize,
    /// Текущая кисть (0/1/2)
    pub brush: u8,
    /// Использовать этот грид при переходе в Playing
    pub active: bool,
    /// Последняя закрашенная ячейка (для поддержки drag)
    pub last_painted: Option<(usize, usize)>,
    /// Нужно ли полностью пересоздать визуал (при смене кол-ва рядов / загрузке)
    pub needs_redraw: bool,
    /// Хэндлы материалов ячеек — cell_materials[row * COLS + col]
    pub cell_materials: Vec<Handle<ColorMaterial>>,
}

impl Default for EditorData {
    fn default() -> Self {
        let rows = 5;
        Self {
            grid: vec![vec![0u8; EDITOR_COLS]; rows],
            rows,
            brush: 1,
            active: false,
            last_painted: None,
            needs_redraw: false,
            cell_materials: Vec::new(),
        }
    }
}

impl EditorData {
    /// Позиция центра ячейки в мировых координатах
    pub fn cell_world_pos(row: usize, col: usize) -> Vec2 {
        let total_w =
            EDITOR_COLS as f32 * BRICK_WIDTH + (EDITOR_COLS - 1) as f32 * BRICK_GAP;
        let start_x = -total_w / 2.0 + BRICK_WIDTH / 2.0;
        let step_x = BRICK_WIDTH + BRICK_GAP;
        let step_y = BRICK_HEIGHT + BRICK_GAP;
        Vec2::new(
            start_x + col as f32 * step_x,
            BRICKS_TOP_Y - row as f32 * step_y,
        )
    }

    /// Преобразует мировую позицию в индекс (row, col), если попали в ячейку
    pub fn world_to_cell(world: Vec2, rows: usize) -> Option<(usize, usize)> {
        let total_w =
            EDITOR_COLS as f32 * BRICK_WIDTH + (EDITOR_COLS - 1) as f32 * BRICK_GAP;
        let start_x = -total_w / 2.0 + BRICK_WIDTH / 2.0;
        let step_x = BRICK_WIDTH + BRICK_GAP;
        let step_y = BRICK_HEIGHT + BRICK_GAP;

        let col_f = (world.x - (start_x - BRICK_WIDTH / 2.0)) / step_x;
        let row_f = ((BRICKS_TOP_Y + BRICK_HEIGHT / 2.0) - world.y) / step_y;

        let col = col_f.floor() as i32;
        let row = row_f.floor() as i32;

        if col < 0 || col >= EDITOR_COLS as i32 || row < 0 || row >= rows as i32 {
            return None;
        }

        // Убеждаемся, что курсор внутри ячейки, а не в зазоре
        let center = Self::cell_world_pos(row as usize, col as usize);
        if (world.x - center.x).abs() <= BRICK_WIDTH / 2.0
            && (world.y - center.y).abs() <= BRICK_HEIGHT / 2.0
        {
            Some((row as usize, col as usize))
        } else {
            None
        }
    }

    /// Сохранить в файл
    pub fn save(&self) {
        let mut s = format!("{} {}\n", EDITOR_COLS, self.rows);
        for row in &self.grid {
            let line: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            s.push_str(&line.join(" "));
            s.push('\n');
        }
        let _ = std::fs::write(EDITOR_FILE, s);
    }

    /// Загрузить из файла (обновляет self.grid и self.rows)
    pub fn load(&mut self) {
        let Ok(text) = std::fs::read_to_string(EDITOR_FILE) else {
            return;
        };
        let mut lines = text.lines();
        let Some(first) = lines.next() else { return };
        let parts: Vec<usize> = first
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() < 2 {
            return;
        }
        let new_rows = parts[1].clamp(EDITOR_MIN_ROWS, EDITOR_MAX_ROWS);

        let mut grid: Vec<Vec<u8>> = lines
            .take(new_rows)
            .map(|line| {
                let mut row: Vec<u8> = line
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .take(EDITOR_COLS)
                    .collect();
                row.resize(EDITOR_COLS, 0);
                row
            })
            .collect();

        if grid.is_empty() {
            return;
        }
        grid.resize(new_rows, vec![0u8; EDITOR_COLS]);
        self.grid = grid;
        self.rows = new_rows;
        self.needs_redraw = true;
    }

    pub fn add_row(&mut self) {
        if self.rows < EDITOR_MAX_ROWS {
            self.grid.push(vec![0u8; EDITOR_COLS]);
            self.rows += 1;
            self.needs_redraw = true;
        }
    }

    pub fn remove_row(&mut self) {
        if self.rows > EDITOR_MIN_ROWS {
            self.grid.pop();
            self.rows -= 1;
            self.needs_redraw = true;
        }
    }
}

/// Цвет ячейки по типу
pub fn editor_cell_color(cell_type: u8) -> Color {
    match cell_type {
        1 => Color::srgb(0.2, 0.6, 0.9),
        2 => Color::srgb(0.9, 0.2, 0.2),
        _ => Color::srgba(0.15, 0.15, 0.2, 1.0), // пустая ячейка — тёмный фон
    }
}
