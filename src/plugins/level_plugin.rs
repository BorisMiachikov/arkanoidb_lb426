use bevy::prelude::*;

use crate::resources::game_state::GameState;
use crate::setup::camera::spawn_camera;
use crate::setup::level::spawn_level_entities;

/// Плагин: загрузка уровней, спавн сущностей
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        // Камера создаётся один раз при старте
        app.add_systems(Startup, spawn_camera);

        // Сущности уровня создаются при входе в состояние Playing
        app.add_systems(OnEnter(GameState::Playing), spawn_level_entities);

        // TODO (Этап 7): OnExit(Playing) → очистка сущностей уровня
    }
}
