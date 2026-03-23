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
