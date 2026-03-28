use bevy::prelude::*;

use crate::resources::level_data::LEVELS;
use crate::setup::level::{
    decode_cell, BRICK_COLORS, BRICK_GAP, BRICK_HEIGHT, BRICK_WIDTH, BRICKS_TOP_Y,
};

pub const EDITOR_COLS: usize = 10;
pub const EDITOR_MIN_ROWS: usize = 1;
pub const EDITOR_MAX_ROWS: usize = 12;

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
    /// Текущий редактируемый уровень (1-based)
    pub current_level: usize,
    /// Общее количество уровней (встроенные + пользовательские файлы)
    pub total_levels: usize,
}

impl Default for EditorData {
    fn default() -> Self {
        let total_levels = Self::discover_total_levels();
        let config = &LEVELS[0];
        let data_rows = config.grid.len().clamp(EDITOR_MIN_ROWS, EDITOR_MAX_ROWS);
        let mut grid: Vec<Vec<u8>> =
            config.grid.iter().take(data_rows).map(|r| r.to_vec()).collect();
        grid.resize(EDITOR_MAX_ROWS, vec![0u8; EDITOR_COLS]);
        let rows = EDITOR_MAX_ROWS;
        Self {
            grid,
            rows,
            brush: 1,
            active: false,
            current_level: 1,
            total_levels,
            last_painted: None,
            needs_redraw: false,
            cell_materials: Vec::new(),
        }
    }
}

impl EditorData {
    /// Подсчёт уровней: встроенные + файлы level_N.lvl, идущие подряд без пропусков
    fn discover_total_levels() -> usize {
        let mut total = LEVELS.len();
        loop {
            let next = total + 1;
            if std::path::Path::new(&format!("levels/level_{}.lvl", next)).exists() {
                total = next;
            } else {
                break;
            }
        }
        total
    }

    /// Путь к файлу для уровня N
    fn level_file(level: usize) -> String {
        format!("levels/level_{}.lvl", level)
    }

    /// Метка текущего уровня для UI
    pub fn level_label(&self) -> String {
        format!("LEVEL {} / {}", self.current_level, self.total_levels)
    }

    /// Переключить уровень на delta (+1 вперёд, -1 назад), с clamp по границам
    pub fn switch_level(&mut self, delta: i32) {
        let new = (self.current_level as i32 + delta)
            .clamp(1, self.total_levels as i32) as usize;
        if new != self.current_level {
            self.current_level = new;
            self.load();
        }
    }

    /// Создать новый пустой уровень за последним
    pub fn new_level(&mut self) {
        self.total_levels += 1;
        self.current_level = self.total_levels;
        self.rows = EDITOR_MAX_ROWS;
        self.grid = vec![vec![0u8; EDITOR_COLS]; self.rows];
        self.needs_redraw = true;
    }

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

        let center = Self::cell_world_pos(row as usize, col as usize);
        if (world.x - center.x).abs() <= BRICK_WIDTH / 2.0
            && (world.y - center.y).abs() <= BRICK_HEIGHT / 2.0
        {
            Some((row as usize, col as usize))
        } else {
            None
        }
    }

    /// Сохранить текущий уровень в файл
    pub fn save(&self) {
        let path = Self::level_file(self.current_level);
        let mut s = format!("{} {}\n", EDITOR_COLS, self.rows);
        for row in &self.grid {
            let line: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            s.push_str(&line.join(" "));
            s.push('\n');
        }
        let _ = std::fs::write(&path, s);
    }

    /// Загрузить текущий уровень: сначала из файла, затем из встроенных данных
    pub fn load(&mut self) {
        let path = Self::level_file(self.current_level);
        if let Ok(text) = std::fs::read_to_string(&path) {
            if self.parse_file_str(&text) {
                return;
            }
        }
        self.load_from_static();
    }

    /// Загрузить из встроенных данных LEVELS (fallback для уровней 1..=5)
    fn load_from_static(&mut self) {
        if self.current_level >= 1 {
            let idx = self.current_level - 1;
            if idx < LEVELS.len() {
                let config = &LEVELS[idx];
                let data_rows = config.grid.len().clamp(EDITOR_MIN_ROWS, EDITOR_MAX_ROWS);
                let mut grid: Vec<Vec<u8>> =
                    config.grid.iter().take(data_rows).map(|r| r.to_vec()).collect();
                // Дополняем пустыми рядами до EDITOR_MAX_ROWS
                grid.resize(EDITOR_MAX_ROWS, vec![0u8; EDITOR_COLS]);
                self.rows = EDITOR_MAX_ROWS;
                self.grid = grid;
                self.needs_redraw = true;
                return;
            }
        }
        // Пользовательские уровни без файла — полностью пустая сетка
        self.rows = EDITOR_MAX_ROWS;
        self.grid = vec![vec![0u8; EDITOR_COLS]; self.rows];
        self.needs_redraw = true;
    }

    /// Разобрать содержимое файла и обновить grid/rows. Возвращает true при успехе.
    fn parse_file_str(&mut self, text: &str) -> bool {
        let mut lines = text.lines();
        let Some(first) = lines.next() else { return false };
        let parts: Vec<usize> = first
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() < 2 {
            return false;
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
            return false;
        }
        grid.resize(new_rows, vec![0u8; EDITOR_COLS]);
        // Дополняем до EDITOR_MAX_ROWS пустыми рядами
        grid.resize(EDITOR_MAX_ROWS, vec![0u8; EDITOR_COLS]);
        self.grid = grid;
        self.rows = EDITOR_MAX_ROWS;
        self.needs_redraw = true;
        true
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

/// Цвет ячейки в редакторе.
/// Normal — полный цвет, Strong — осветлённый (смесь с белым).
pub fn editor_cell_color(cell: u8) -> Color {
    match decode_cell(cell) {
        None => Color::srgba(0.15, 0.15, 0.2, 1.0), // пустая
        Some((brick_type, ci)) => {
            let c = BRICK_COLORS[ci].to_srgba();
            if brick_type == 2 {
                // Strong: смешиваем с белым, чтобы визуально отличался
                Color::srgb(
                    (c.red   * 0.5 + 0.5_f32).min(1.0),
                    (c.green * 0.5 + 0.5_f32).min(1.0),
                    (c.blue  * 0.5 + 0.5_f32).min(1.0),
                )
            } else {
                BRICK_COLORS[ci]
            }
        }
    }
}
