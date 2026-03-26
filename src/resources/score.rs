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

/// Ресурс: выбранный пункт главного меню (0=Play, 1=LevelEditor)
#[derive(Resource, Default)]
pub struct MenuSelection(pub usize);

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
