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
