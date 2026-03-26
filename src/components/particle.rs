use bevy::prelude::*;

/// Компонент частицы эффекта — убивается по таймеру, затухает
#[derive(Component)]
pub struct Particle {
    pub lifetime: Timer,
}
