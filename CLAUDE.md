# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Проект

Клон игры Arkanoid/Breakout на Rust с использованием движка Bevy 0.15. Архитектура — ECS (Entity-Component-System). Проект следует плану из `project.md` и поэтапной дорожной карте из `Roadmap.md`.

## Команды

```bash
# Первичная инициализация (если ещё не создан cargo-проект)
cargo new arkanoid_lb426
cargo add bevy@0.15

# Сборка и запуск
cargo run
cargo build --release

# Быстрая сборка для разработки (Bevy dynamic linking)
cargo run --features bevy/dynamic_linking

# Проверка и тесты
cargo check
cargo test
cargo clippy
```

## Архитектура

Проект разбит на **плагины** — каждый плагин отвечает за свою область:

| Плагин | Ответственность |
|--------|----------------|
| `GameplayPlugin` | Правила игры, жизни, победа/поражение |
| `PhysicsPlugin` | Движение сущностей и AABB-коллизии |
| `UIPlugin` | Счёт, уровень, жизни, активные бонусы |
| `LevelPlugin` | Загрузка и прогрессия уровней |

### Структура `src/`

```
src/
├── main.rs               # Точка входа, регистрация плагинов
├── app.rs                # Настройка App
├── plugins/              # Bevy-плагины
├── components/           # ECS-компоненты (только данные, без логики)
│   ├── paddle.rs, ball.rs, brick.rs
│   ├── velocity.rs, collider.rs
│   └── bonus.rs, ufo.rs, bomb.rs
├── systems/              # Системы (чистая логика, без состояния)
│   ├── input.rs, movement.rs, collision.rs
│   └── gameplay.rs, bonus.rs, ufo.rs
├── resources/            # Глобальные ресурсы
│   ├── game_state.rs
│   └── score.rs
└── setup/                # Инициализация сцены
    ├── camera.rs
    └── level.rs
```

### Игровые состояния

```
MainMenu → Playing ↔ Paused → GameOver / LevelComplete
```

### Физика

Используются **только кастомные AABB-коллизии** — без сторонних физических библиотек (Rapier и т.п.).

Проверяемые пары коллизий:
- Ball ↔ Paddle, Ball ↔ Brick, Ball ↔ Wall, Ball ↔ UFO
- Bonus ↔ Paddle (подбор), Bomb ↔ Paddle (урон)

### Управление

- **A / ←** — ракетка влево
- **D / →** — ракетка вправо
- **Пробел** — стрельба (если активен бонус-пулемёт)

## Принципы разработки

- **Компоненты** хранят только данные, без логики
- **Системы** содержат логику, без состояния
- Минимальная связанность — взаимодействие только через ECS (Query, Res)
- Следовать этапам из `Roadmap.md` — каждый этап имеет конкретный результат
