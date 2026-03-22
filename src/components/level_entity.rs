use bevy::prelude::*;

/// Маркер: сущность принадлежит текущему уровню.
/// При рестарте все LevelEntity удаляются.
#[derive(Component)]
pub struct LevelEntity;
