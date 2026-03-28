/// Конфигурация уровня
pub struct LevelConfig {
    pub ball_speed_multiplier: f32,
    /// Сетка блоков: кодировка — 0=пусто, 1-6=Normal(цвет), 7-12=Strong(цвет)
    /// Цвета: 0=Blue,1=Green,2=Yellow,3=Orange,4=Purple,5=Red
    pub grid: &'static [&'static [u8]],
    /// Позиции спавна НЛО (x, y)
    pub ufos: &'static [(f32, f32)],
    /// Скорость НЛО
    pub ufo_speed: f32,
    /// Интервал сброса бомб (сек)
    pub ufo_bomb_interval: f32,
}

pub const LEVELS: &[LevelConfig] = &[
    // Уровень 1: классический — без НЛО
    LevelConfig {
        ball_speed_multiplier: 1.0,
        grid: &[
            &[7, 7, 7, 7, 7, 7, 7, 7, 7, 7],  // Strong Blue
            &[2, 2, 2, 2, 2, 2, 2, 2, 2, 2],  // Normal Green
            &[3, 3, 3, 3, 3, 3, 3, 3, 3, 3],  // Normal Yellow
            &[4, 4, 4, 4, 4, 4, 4, 4, 4, 4],  // Normal Orange
            &[5, 5, 5, 5, 5, 5, 5, 5, 5, 5],  // Normal Purple
        ],
        ufos: &[],
        ufo_speed: 0.0,
        ufo_bomb_interval: 0.0,
    },
    // Уровень 2: шахматный паттерн + 1 НЛО
    LevelConfig {
        ball_speed_multiplier: 1.25,
        grid: &[
            &[ 7,  7,  7,  7,  7,  7,  7,  7,  7,  7],  // Strong Blue
            &[ 8,  0,  8,  0,  8,  0,  8,  0,  8,  0],  // Strong Green / пусто
            &[ 3,  9,  3,  9,  3,  9,  3,  9,  3,  9],  // Normal Yellow / Strong Yellow
            &[10,  4, 10,  4, 10,  4, 10,  4, 10,  4],  // Strong Orange / Normal Orange
            &[ 5,  5,  5,  5,  5,  5,  5,  5,  5,  5],  // Normal Purple
            &[ 6,  6,  6,  6,  6,  6,  6,  6,  6,  6],  // Normal Red
        ],
        ufos: &[(0.0, -30.0)],
        ufo_speed: 130.0,
        ufo_bomb_interval: 4.0,
    },
    // Уровень 3: пирамида + 2 НЛО
    LevelConfig {
        ball_speed_multiplier: 1.5,
        grid: &[
            &[ 7,  7,  7,  7,  7,  7,  7,  7,  7,  7],
            &[ 8,  8,  8,  8,  8,  8,  8,  8,  8,  8],
            &[ 0,  9,  9,  9,  9,  9,  9,  9,  9,  0],
            &[ 0,  0,  4, 10, 10, 10, 10,  4,  0,  0],
            &[ 0,  0,  0,  5, 11, 11,  5,  0,  0,  0],
            &[ 0,  0,  0,  0,  6,  6,  0,  0,  0,  0],
        ],
        ufos: &[(-180.0, -30.0), (180.0, -30.0)],
        ufo_speed: 170.0,
        ufo_bomb_interval: 3.0,
    },
    // Уровень 4: крепость (кольца) + 3 НЛО
    LevelConfig {
        ball_speed_multiplier: 1.75,
        grid: &[
            &[ 7,  7,  7,  7,  7,  7,  7,  7,  7,  7],
            &[ 8,  2,  2,  2,  2,  2,  2,  2,  2,  8],
            &[ 9,  3,  9,  9,  9,  9,  9,  9,  3,  9],
            &[10,  4, 10,  4,  4,  4,  4, 10,  4, 10],
            &[11,  5, 11, 11, 11, 11, 11, 11,  5, 11],
            &[12,  6,  6,  6,  6,  6,  6,  6,  6, 12],
            &[ 7,  7,  7,  7,  7,  7,  7,  7,  7,  7],
        ],
        ufos: &[(-150.0, 230.0), (150.0, 230.0), (0.0, -50.0)],
        ufo_speed: 200.0,
        ufo_bomb_interval: 2.5,
    },
    // Уровень 5: финальная стена + 4 НЛО
    LevelConfig {
        ball_speed_multiplier: 2.0,
        grid: &[
            &[7, 7, 7, 7, 7, 7, 7, 7, 7, 7],
            &[8, 8, 8, 8, 8, 8, 8, 8, 8, 8],
            &[9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
            &[4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            &[11,11,11,11,11,11,11,11,11,11],
            &[12,12,12,12,12,12,12,12,12,12],
            &[ 7, 7, 7, 7, 7, 7, 7, 7, 7, 7],
        ],
        ufos: &[(-180.0, 240.0), (180.0, 240.0), (-120.0, -60.0), (120.0, -60.0)],
        ufo_speed: 220.0,
        ufo_bomb_interval: 2.0,
    },
];
