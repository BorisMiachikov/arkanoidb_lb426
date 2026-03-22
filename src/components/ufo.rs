use bevy::prelude::*;

/// НЛО — горизонтально движущееся препятствие, сбрасывает бомбы
#[derive(Component)]
pub struct Ufo {
    pub speed: f32,
    pub direction: f32,       // 1.0 = вправо, -1.0 = влево
    pub bomb_timer: Timer,
}

impl Ufo {
    pub fn new(speed: f32, bomb_interval_secs: f32) -> Self {
        Self {
            speed,
            direction: 1.0,
            bomb_timer: Timer::from_seconds(bomb_interval_secs, TimerMode::Repeating),
        }
    }
}
