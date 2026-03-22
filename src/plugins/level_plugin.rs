use bevy::prelude::*;

use crate::resources::game_state::GameState;
use crate::setup::camera::spawn_camera;
use crate::setup::level::{cleanup_level, spawn_level_entities};

/// Плагин: загрузка уровней, спавн и очистка сущностей
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        // Камера создаётся один раз при старте
        app.add_systems(Startup, spawn_camera);

        // Сущности уровня создаются при входе в Playing
        app.add_systems(OnEnter(GameState::Playing), spawn_level_entities);

        // Очистка при выходе из Playing (рестарт / победа)
        app.add_systems(OnExit(GameState::Playing), cleanup_level);
    }
}
