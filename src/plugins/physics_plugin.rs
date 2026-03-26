use bevy::prelude::*;

use crate::resources::game_state::GameState;
use crate::resources::score::Paused;
use crate::systems::bonus::{
    apply_ball_grow_system, apply_fireball_effect_system, apply_paddle_grow_system,
    bonus_pickup_system, update_bonus_effects_system,
};
use crate::systems::particles::{ball_trail_system, update_particles_system, BallTrailTimer};
use crate::systems::collision::{
    ball_brick_collision_system, ball_paddle_collision_system, ball_wall_collision_system,
};
use crate::systems::input::{ball_stuck_system, paddle_input_system};
use crate::systems::movement::apply_velocity_system;
use crate::systems::gun::{
    bullet_brick_collision_system, bullet_ufo_collision_system, cleanup_bullets_system,
    fire_gun_system,
};
use crate::systems::ufo::{
    ball_ufo_collision_system, bomb_brick_collision_system, bomb_paddle_collision_system,
    cleanup_fallen_bombs_system, ufo_brick_collision_system, ufo_movement_system,
};

/// Плагин: физика — ввод, движение, коллизии, бонусы
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BallTrailTimer>();

        // Частицы обновляются в Update (не FixedUpdate) — чтобы не превышать лимит chain
        app.add_systems(
            Update,
            update_particles_system.run_if(in_state(GameState::Playing)),
        );

        // След мяча — FixedUpdate, после движения
        app.add_systems(
            FixedUpdate,
            ball_trail_system
                .after(apply_velocity_system)
                .run_if(in_state(GameState::Playing))
                .run_if(|p: Res<Paused>| !p.0),
        );

        app.add_systems(
            FixedUpdate,
            (
                paddle_input_system,
                ball_stuck_system,
                apply_velocity_system,
                ball_wall_collision_system,
                ball_brick_collision_system,
                ball_paddle_collision_system,
                bonus_pickup_system,
                apply_paddle_grow_system,
                apply_ball_grow_system,
                update_bonus_effects_system,
                ufo_movement_system,
                ufo_brick_collision_system,
                ball_ufo_collision_system,
                bomb_brick_collision_system,
                bomb_paddle_collision_system,
                cleanup_fallen_bombs_system,
                fire_gun_system,
                bullet_brick_collision_system,
                bullet_ufo_collision_system,
                cleanup_bullets_system,
            )
                .chain()
                .run_if(in_state(GameState::Playing))
                .run_if(|p: Res<Paused>| !p.0),
        );

        // Цвет мяча при получении FireBallEffect — отдельно, чтобы не превышать лимит цепочки
        app.add_systems(
            FixedUpdate,
            apply_fireball_effect_system
                .after(bonus_pickup_system)
                .run_if(in_state(GameState::Playing))
                .run_if(|p: Res<Paused>| !p.0),
        );
    }
}
