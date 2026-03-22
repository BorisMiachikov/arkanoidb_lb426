use bevy::prelude::*;

/// Состояния игры. Переходы: MainMenu → Playing ↔ Paused → GameOver / LevelComplete
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
    LevelComplete,
}
