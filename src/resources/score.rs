use bevy::prelude::*;

/// Ресурс: текущий счёт игрока
#[derive(Resource, Debug, Default)]
pub struct Score {
    pub value: u32,
}

/// Ресурс: жизни игрока
#[derive(Resource, Debug)]
pub struct Lives {
    pub count: u32,
}

impl Default for Lives {
    fn default() -> Self {
        Self { count: 3 }
    }
}

/// Ресурс: текущий номер уровня
#[derive(Resource, Debug, Default)]
pub struct CurrentLevel {
    pub number: u32,
}
