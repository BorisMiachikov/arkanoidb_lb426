use bevy::prelude::*;

/// Маркер: сущность является ракеткой игрока
#[derive(Component, Debug)]
pub struct Paddle {
    pub speed: f32,
}

impl Default for Paddle {
    fn default() -> Self {
        Self { speed: 400.0 }
    }
}
