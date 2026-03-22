use bevy::prelude::*;

/// Маркер: сущность является мячом
#[derive(Component, Debug)]
pub struct Ball {
    pub radius: f32,
}

impl Default for Ball {
    fn default() -> Self {
        Self { radius: 10.0 }
    }
}
