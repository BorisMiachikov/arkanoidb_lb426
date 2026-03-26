use bevy::prelude::*;

/// Активный эффект: ракетка увеличена в 1.5 раза
#[derive(Component)]
pub struct PaddleGrowEffect {
    pub timer: Timer,
    pub original_half_width: f32,
}

/// Активный эффект: липкая ракетка — мяч прилипает при касании
#[derive(Component)]
pub struct StickyEffect {
    pub timer: Timer,
}

/// Активный эффект: мяч увеличен в 1.5 раза
#[derive(Component)]
pub struct BallGrowEffect {
    pub timer: Timer,
    pub original_half_size: f32,
}

/// Активный эффект: ракетка стреляет снарядами (LeftCtrl / RightCtrl)
#[derive(Component)]
pub struct GunPaddleEffect {
    /// Общая длительность эффекта
    pub timer: Timer,
    /// Кулдаун между выстрелами
    pub fire_rate: Timer,
}

/// Активный эффект: огненный мяч — пробивает кирпичи насквозь без отскока
#[derive(Component)]
pub struct FireBallEffect {
    pub timer: Timer,
}
