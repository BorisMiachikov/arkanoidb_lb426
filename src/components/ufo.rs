use bevy::prelude::*;

/// Маркер: сущность является НЛО (динамическое препятствие)
#[derive(Component, Debug)]
pub struct Ufo {
    pub speed: f32,
}

impl Default for Ufo {
    fn default() -> Self {
        Self { speed: 150.0 }
    }
}
