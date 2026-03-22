use bevy::prelude::*;

/// AABB-коллайдер: размер ограничивающего прямоугольника
#[derive(Component, Debug, Clone, Copy)]
pub struct Collider {
    pub half_width: f32,
    pub half_height: f32,
}

impl Collider {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            half_width: width / 2.0,
            half_height: height / 2.0,
        }
    }
}
