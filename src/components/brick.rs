use bevy::prelude::*;

/// Тип блока
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrickType {
    Normal, // 1 удар
    Strong, // несколько ударов
}

/// Маркер: сущность является блоком
#[derive(Component, Debug)]
pub struct Brick {
    pub brick_type: BrickType,
    pub health: u32,
    pub score_value: u32,
}

impl Default for Brick {
    fn default() -> Self {
        Self {
            brick_type: BrickType::Normal,
            health: 1,
            score_value: 100,
        }
    }
}
