use bevy::prelude::*;

/// Тип бонуса
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BonusType {
    PaddleGrow,   // Увеличение ракетки
    StickyPaddle, // Липкая ракетка
    GunPaddle,    // Пулемёт
    BallGrow,     // Увеличение мяча
    FireBall,     // Огненный мяч — пробивает кирпичи насквозь
    MultiBall,    // Тройной мяч — делится на 3
}

/// Маркер: сущность является бонусом (падает вниз при разрушении блока)
#[derive(Component, Debug)]
pub struct Bonus {
    pub bonus_type: BonusType,
}
