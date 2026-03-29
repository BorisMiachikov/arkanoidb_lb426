use bevy::prelude::*;

/// Звуковые события игры — системы отправляют, AssetPlugin воспроизводит
#[derive(Message, Clone, Copy, PartialEq, Eq)]
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
