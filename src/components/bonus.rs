use bevy::prelude::*;

/// Тип бонуса
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BonusType {
    PaddleGrow,   // Увеличение ракетки
    StickyPaddle, // Липкая ракетка
    GunPaddle,    // Пулемёт
    BallGrow,     // Увеличение мяча
}

/// Маркер: сущность является бонусом (падает вниз при разрушении блока)
#[derive(Component, Debug)]
pub struct Bonus {
    pub bonus_type: BonusType,
}
