use bevy::prelude::*;

/// Маркер: сущность является бомбой (сбрасывается НЛО, падает вниз)
#[derive(Component, Debug)]
pub struct Bomb {
    pub damage: u32,
}

impl Default for Bomb {
    fn default() -> Self {
        Self { damage: 1 }
    }
}
