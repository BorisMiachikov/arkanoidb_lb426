use bevy::prelude::*;

/// Звуковые события игры — системы отправляют, AssetPlugin воспроизводит
#[derive(Event, Clone, Copy, PartialEq, Eq)]
pub enum SoundEvent {
    BallHitWall,
    BallHitPaddle,
    BallHitBrick,
    BrickBreak,
    BonusPickup,
    LifeLost,
    GameOver,
    BulletFire,
    UfoHit,
    BombHit,
}
