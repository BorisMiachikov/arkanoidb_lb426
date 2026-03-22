use bevy::prelude::*;

use crate::resources::game_state::GameState;
use crate::systems::collision::{
    ball_brick_collision_system, ball_paddle_collision_system, ball_wall_collision_system,
};
use crate::systems::input::paddle_input_system;
use crate::systems::movement::apply_velocity_system;

/// Плагин: физика — ввод, движение, AABB-коллизии
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                paddle_input_system,
                apply_velocity_system,
                ball_wall_collision_system,
                ball_brick_collision_system,
                ball_paddle_collision_system,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}
