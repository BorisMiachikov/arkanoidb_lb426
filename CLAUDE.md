# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Проект

Клон игры Arkanoid/Breakout на Rust с использованием движка Bevy 0.15. Архитектура — ECS (Entity-Component-System). Проект следует плану из `project.md` и поэтапной дорожной карте из `Roadmap.md`.

## Команды

```bash
# Сборка и запуск
cargo run
cargo build --release

# Быстрая сборка для разработки
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
| `AssetPlugin`    | Загрузка ассетов, музыка меню/геймплея, воспроизведение SoundEvent |
| `GameplayPlugin` | Правила игры, жизни, победа/поражение, пауза, отладочные клавиши |
| `PhysicsPlugin` | Движение, ввод и все AABB-коллизии (включая пулемёт), частицы |
| `UiPlugin` | HUD (счёт, уровень, жизни, бонусы, рекорд), экраны состояний |
| `LevelPlugin` | Камера, загрузка и очистка уровней |
| `EditorPlugin` | Редактор уровней — мышь + клавиатура, сохранение/загрузка custom_level.lvl |

### Структура `src/`

```
src/
├── main.rs               # Точка входа
├── app.rs                # Настройка App, плагины, ClearColor
├── events.rs             # SoundEvent (10 вариантов) — системы пишут, AssetPlugin играет
├── plugins/              # Bevy-плагины
├── components/           # ECS-компоненты (только данные, без логики)
│   ├── ball.rs           # Ball { radius }, BallStuck (маркер запуска)
│   ├── brick.rs          # Brick { brick_type, health, score_value }
│   ├── bonus.rs          # Bonus { bonus_type: BonusType } — 7 типов бонусов
│   ├── bonus_effects.rs  # PaddleGrowEffect, StickyEffect, BallGrowEffect,
│   │                     # GunPaddleEffect, FireBallEffect
│   ├── bomb.rs           # Bomb { damage }
│   ├── bullet.rs         # Bullet (маркер снаряда пулемёта)
│   ├── particle.rs       # Particle { lifetime: Timer } — частицы эффектов
│   ├── ufo.rs            # Ufo { speed, direction, bomb_timer, health }
│   ├── paddle.rs         # Paddle { speed }
│   ├── velocity.rs       # Velocity { x, y }
│   ├── collider.rs       # Collider { half_width, half_height }
│   ├── level_entity.rs   # LevelEntity (маркер для очистки уровня)
│   └── wall.rs           # Wall (маркер)
├── systems/              # Системы (чистая логика)
│   ├── input.rs          # paddle_input_system, ball_stuck_system
│   ├── movement.rs       # apply_velocity_system
│   ├── collision.rs      # ball_wall/brick/paddle коллизии, дроп бонусов
│   ├── bonus.rs          # подбор бонусов, применение/откат эффектов, MultiBall спавн
│   ├── gun.rs            # fire_gun_system, bullet коллизии, cleanup
│   ├── particles.rs      # update_particles_system, ball_trail_system, spawn_burst, BallTrailTimer
│   ├── editor.rs         # EditorCell, EditorEntity, setup/cleanup_editor,
│   │                     # editor_mouse/keyboard/redraw_system
│   ├── ufo.rs            # движение НЛО, коллизии с блоками, бомбы
│   └── gameplay.rs       # потеря мяча, победа, GameOver, debug skip, пауза
├── plugins/              # Bevy-плагины
│   ├── asset_plugin.rs   # AssetPlugin: load_assets, MenuMusicController, MusicController
│   ├── editor_plugin.rs  # EditorPlugin — LevelEditor состояние
│   ├── gameplay_plugin.rs
│   ├── level_plugin.rs
│   ├── physics_plugin.rs
│   └── ui_plugin.rs
├── resources/            # Глобальные ресурсы
│   ├── assets.rs         # GameAssets: Handle<AudioSource> ×11, Handle<Image> ×15
│   ├── game_state.rs     # GameState enum (+ Options, LevelEditor)
│   ├── score.rs          # Score, Lives, CurrentLevel, BallSpeedMultiplier,
│   │                     # DebugSkipPending, Paused, MenuSelection, OptionsSelection,
│   │                     # HighScore, AudioSettings { music_volume, sfx_volume }
│   ├── editor.rs         # EditorData, EDITOR_COLS/MIN_ROWS/MAX_ROWS/FILE,
│   │                     # editor_cell_color — логика редактора и файл custom_level.lvl
│   └── level_data.rs     # LevelConfig, LEVELS (статические данные уровней)
└── setup/                # Инициализация сцены
    ├── camera.rs         # spawn_camera
    └── level.rs          # spawn_level_entities, cleanup_level, константы окна
```

### Игровые состояния

```
Startup → MainMenu ──────────────────────────── LevelEditor
               │  └─ Options (ESC → MainMenu)        │ ESC → MainMenu
               │                                     │ P   → Playing (кастомный уровень)
               │ Play Game (Enter/Space)
               ↓
           Playing → LevelComplete → Playing (следующий уровень)
                  ↘ GameOver → Playing (Enter/Space, рестарт)
                            ↘ MainMenu  (ESC, сброс счёта/уровня)
```

**Пауза** реализована через ресурс `Paused(bool)` — не меняет `GameState`,
поэтому уровень остаётся нетронутым. Физика и геймплей проверяют `.run_if(|p: Res<Paused>| !p.0)`.

### Физика

Только **кастомные AABB-коллизии** — без Rapier и других физических библиотек.

Пары коллизий:
- Ball ↔ Wall, Ball ↔ Brick, Ball ↔ Paddle, Ball ↔ UFO
- UFO ↔ Brick (НЛО не проходит сквозь блоки)
- Bonus ↔ Paddle (подбор), Bomb ↔ Paddle (урон), Bomb ↔ Brick (бомба исчезает)
- Bullet ↔ Brick (урон блоку), Bullet ↔ UFO (урон НЛО)

### Механики

- **BallStuck** — компонент-маркер: мяч прилипает к ракетке (при старте и после потери жизни). Запуск: Пробел или движение ракетки.
- **LevelEntity** — все сущности уровня помечаются этим маркером. `OnExit(Playing)` → `cleanup_level` удаляет всё.
- **НЛО** — уничтожаются за 2 удара, респавнятся случайно **выше блоков** `y ∈ [200, 270]` или **ниже блоков** `y ∈ [−120, 0]`.
- **Бонусные эффекты** — применяются через `Added<T>`, откатываются по таймеру через `remove::<T>()`.
- **GunPaddleEffect** — содержит два таймера: `timer` (длительность 15 сек) и `fire_rate` (0.18 сек между выстрелами).
- **DebugSkipPending** — флаг-ресурс для двухшагового перехода Playing → LevelComplete → Playing при нажатии `*`.
- **Границы ракетки** — вычисляются динамически из `Collider.half_width`, корректно работают при PaddleGrow.
- **MusicController** — маркер геймплейной музыки, не удаляется между уровнями. Стоп: `OnEnter(MainMenu)`.
- **MenuMusicController** — маркер музыки меню. Старт: `OnEnter(MainMenu)`, стоп: `OnExit(MainMenu)`.
- **SoundEvent** — отправляется из систем через `EventWriter<SoundEvent>`, воспроизводится в `AssetPlugin` через `AudioPlayer + PlaybackSettings::DESPAWN`.
- **AudioSettings** — ресурс `{ music_volume, sfx_volume }` (0.0–1.0). Музыка: `sink.set_volume()` каждый кадр. SFX: `PlaybackSettings { volume: Volume::new(sfx_volume), ..PlaybackSettings::DESPAWN }`. Изменяется в Options.
- **OptionsSelection** — ресурс навигации внутри экрана Options (0=Music, 1=SFX, 2=Back), сбрасывается при `OnEnter(Options)`.
- **LivesContainer / LivesIcon** — жизни в HUD как `Node + BackgroundColor` иконки. Обновляются через `despawn_descendants() + with_children()` при `lives.is_changed()`.
- **ExtraLife** — 7-й тип бонуса (розовый): +1 жизнь, макс. 9. Обрабатывается прямо в `bonus_pickup_system` через `ResMut<Lives>`.

### Уровни (`level_data.rs`)

Уровень описывается `LevelConfig` с полями:
- `grid: &[&[u8]]` — сетка блоков (0 = пусто, 1 = Normal, 2 = Strong)
- `ball_speed_multiplier: f32`
- `ufos: &[(f32, f32)]` — позиции спавна НЛО
- `ufo_speed: f32`, `ufo_bomb_interval: f32`

### Управление (Playing)

| Клавиша | Действие |
|---------|----------|
| A / ← | ракетка влево |
| D / → | ракетка вправо |
| Пробел | запуск мяча |
| Ctrl (Left/Right) | стрельба пулемётом (если активен GunPaddle) |
| Escape | пауза; повторно → выход в MainMenu |
| `*` Numpad | **[DEBUG]** следующий уровень |

### Управление (GameOver)

| Клавиша | Действие |
|---------|----------|
| Enter / Space | рестарт с первого уровня |
| ESC | выход в MainMenu (счёт/уровень/жизни сбрасываются) |

### Управление (LevelEditor)

| Клавиша | Действие |
|---------|----------|
| ЛКМ / drag | нарисовать ячейку выбранной кистью |
| ПКМ / drag | стереть ячейку (тип 0) |
| 0 / 1 / 2 | выбрать кисть (пусто / Normal / Strong) |
| + / NumpadAdd | добавить ряд (макс. 10) |
| - / NumpadSubtract | убрать ряд (мин. 1) |
| S | сохранить в `custom_level.lvl` |
| L | загрузить из `custom_level.lvl` |
| P | перейти в Playing с кастомным уровнем |
| Escape | вернуться в MainMenu |

### Важные константы (`setup/level.rs`)

```rust
WINDOW_WIDTH = 800.0, WINDOW_HEIGHT = 600.0
HALF_W = 400.0, HALF_H = 300.0
WALL_THICKNESS = 16.0
PADDLE_WIDTH = 120.0, PADDLE_Y = -250.0
BALL_SIZE = 20.0, BALL_INITIAL_VX = 200.0, BALL_INITIAL_VY = 350.0
BRICK_WIDTH = 72.0, BRICK_HEIGHT = 24.0
BRICKS_TOP_Y = 170.0  (центр верхнего ряда; блоки занимают y ∈ [18, 182])
```

## Ассеты

Структура `assets/` (отсутствующие файлы не крашат игру — Bevy загружает асинхронно):

```
assets/
├── music/    menu.ogg, gameplay.ogg
├── sounds/   ball_hit.ogg, brick_hit.ogg, brick_break.ogg, bonus_pickup.ogg,
│             life_lost.ogg, game_over.ogg, bullet_fire.ogg, ufo_hit.ogg, bomb_hit.ogg
└── sprites/  padle.png (⚠ опечатка в имени файла — именно так, не paddle.png),
              ball.png, ball_fire.png, brick_normal.png, brick_strong.png,
              brick_strong_hit.png, ufo.png, bullet.png, bomb.png, bonus_*.png ×6
```

**Спрайты vs Mesh2d:** ракетка и блоки (Normal/Strong/Strong-hit) используют `Sprite`. Мяч, стены, НЛО, бонусы, пуля, бомба, частицы — `Mesh2d`.

**UI-изображения:** в Bevy 0.15 UI использовать `ImageNode`, не `Sprite`.

**Шрифт:** Bevy дефолтный шрифт поддерживает только ASCII — весь UI-текст должен быть на латинице. Для Кириллицы нужен TTF-файл в `assets/fonts/` с явной передачей в `TextFont { font: asset_server.load("fonts/..."), .. }`.

**Выход из приложения:** `use bevy::app::AppExit;` → `EventWriter<AppExit>` → `app_exit.send(AppExit::Success)`.

## Принципы разработки

- **Компоненты** хранят только данные, без логики
- **Системы** содержат логику, без состояния
- Минимальная связанность — взаимодействие только через ECS (Query, Res)
- Следовать этапам из `Roadmap.md` — каждый этап имеет конкретный результат
- При конфликте Bevy-запросов (B0001) использовать `Without<T>` фильтры
