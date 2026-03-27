use bevy::prelude::*;

/// Ресурс: текущий счёт игрока
#[derive(Resource, Debug, Default)]
pub struct Score {
    pub value: u32,
}

/// Ресурс: жизни игрока
#[derive(Resource, Debug)]
pub struct Lives {
    pub count: u32,
}

impl Default for Lives {
    fn default() -> Self {
        Self { count: 3 }
    }
}

/// Ресурс: текущий номер уровня
#[derive(Resource, Debug, Default)]
pub struct CurrentLevel {
    pub number: u32,
}

/// Ресурс: флаг отладочного пропуска уровня (автоматический переход из LevelComplete)
#[derive(Resource, Default)]
pub struct DebugSkipPending(pub bool);

/// Ресурс: игра на паузе (не меняет GameState)
#[derive(Resource, Default)]
pub struct Paused(pub bool);

/// Ресурс: множитель скорости мяча для текущего уровня
#[derive(Resource, Debug)]
pub struct BallSpeedMultiplier(pub f32);

impl Default for BallSpeedMultiplier {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Ресурс: выбранный пункт главного меню (0=Play, 1=LevelEditor, 2=Options, 3=Quit)
#[derive(Resource, Default)]
pub struct MenuSelection(pub usize);

/// Ресурс: выбранный пункт в Options (0=Music, 1=SFX, 2=Back)
#[derive(Resource, Default)]
pub struct OptionsSelection(pub usize);

/// Ресурс: настройки громкости
#[derive(Resource)]
pub struct AudioSettings {
    pub music_volume: f32, // 0.0 – 1.0
    pub sfx_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self { music_volume: 0.7, sfx_volume: 0.8 }
    }
}

/// Одна запись таблицы рекордов
#[derive(Clone, Debug)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
}

const MAX_SCORES: usize = 10;
const SCORES_FILE: &str = "scores.dat";

/// Ресурс: таблица рекордов (топ-10), сохраняется в scores.dat
#[derive(Resource, Default)]
pub struct ScoreTable {
    pub entries: Vec<ScoreEntry>,
}

impl ScoreTable {
    /// Загружает таблицу из файла; формат строки: `score,name`
    pub fn load() -> Self {
        let entries = std::fs::read_to_string(SCORES_FILE)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ',');
                let score: u32 = parts.next()?.trim().parse().ok()?;
                let name = parts.next()?.trim().to_string();
                if name.is_empty() { return None; }
                Some(ScoreEntry { name, score })
            })
            .collect();
        Self { entries }
    }

    /// Сохраняет таблицу в файл
    pub fn save(&self) {
        let content = self.entries
            .iter()
            .map(|e| format!("{},{}", e.score, e.name))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = std::fs::write(SCORES_FILE, content);
    }

    /// Возвращает true если очки попадают в топ-10
    pub fn qualifies(&self, score: u32) -> bool {
        score > 0
            && (self.entries.len() < MAX_SCORES
                || score > self.entries.last().map_or(0, |e| e.score))
    }

    /// Добавляет запись, сортирует и обрезает до MAX_SCORES
    pub fn add(&mut self, entry: ScoreEntry) {
        self.entries.push(entry);
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        self.entries.truncate(MAX_SCORES);
    }
}

/// Ресурс: буфер ввода имени игрока при попадании в таблицу рекордов
#[derive(Resource, Default)]
pub struct NameInput {
    pub text: String,
}

/// Ресурс: рекорд — сохраняется в файл при обновлении
#[derive(Resource, Debug, Default)]
pub struct HighScore {
    pub value: u32,
}

impl HighScore {
    const FILE: &'static str = "highscore.dat";

    /// Загружает рекорд из файла; возвращает 0 если файл не существует
    pub fn load() -> Self {
        let value = std::fs::read_to_string(Self::FILE)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        Self { value }
    }

    /// Сохраняет рекорд в файл
    pub fn save(&self) {
        let _ = std::fs::write(Self::FILE, self.value.to_string());
    }
}
