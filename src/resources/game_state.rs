use bevy::prelude::*;

/// Состояния игры.
/// Переходы: MainMenu → Playing → GameOver / LevelComplete → Playing
/// Пауза реализована через ресурс Paused (не меняет GameState).
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Options,
    Playing,
    GameOver,
    LevelComplete,
    LevelEditor,
}
