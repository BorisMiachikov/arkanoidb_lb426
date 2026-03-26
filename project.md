# Техническое задание: Игра в жанре Arkanoid/Breakout (Rust + Bevy 0.15)

## 1. Общая информация

**Название проекта:** arkanoidb_lb426
**Жанр:** Аркада / Breakout
**Язык разработки:** Rust (stable)
**Движок:** Bevy 0.15
**Физика:** Кастомная (AABB-коллизии, без Rapier)
**Репозиторий:** https://github.com/BorisMiachikov/arkanoidb_lb426

---

## 2. Описание игры

Игрок управляет ракеткой в нижней части экрана.
Цель — отбивать мяч и уничтожать все блоки на уровне.

Игра заканчивается:
- **Победой** — при уничтожении всех блоков → `LevelComplete`
- **Поражением** — если мяч падает за нижнюю границу при 0 жизнях → `GameOver`

---

## 3. Основной игровой цикл

1. Главное меню (`MainMenu`) → ENTER → `Playing`
2. Инициализация уровня (`OnEnter(Playing)` → `spawn_level_entities`)
3. Мяч появляется прилипшим к ракетке (`BallStuck`)
4. Запуск мяча пробелом или движением ракетки
5. Обработка ввода → движение → коллизии → эффекты
6. Проверка условий победы/поражения
7. Переход к следующему уровню или рестарт

---

## 4. Игровые сущности

### 4.1 Ракетка (Paddle)
- Управляется: A/D или ←/→, скорость 400 px/s
- Границы вычисляются динамически по `Collider.half_width`
- Эффекты: PaddleGrow (×1.5 на 10 сек), StickyPaddle (прилипание на 10 сек), GunPaddle (пулемёт на 15 сек)

### 4.2 Мяч (Ball)
- Размер: 20×20, начальная скорость: vx=200, vy=350 (×`BallSpeedMultiplier`)
- Отскакивает от: стен, ракетки, блоков, НЛО
- Угол отскока от ракетки зависит от точки контакта
- Эффект: BallGrow (×1.5 на 10 сек), FireBall (пробивает блоки 8 сек)
- Скорость растёт +0.5% за каждый удар по блоку, cap 750 px/s

### 4.3 Блоки (Bricks)
- Normal: 1 удар, 100 очков
- Strong: 2 удара, 200 очков
- 30% шанс дропа бонуса при уничтожении

### 4.4 Бонусы (Power-ups)

| Тип | Цвет | Эффект | Длительность |
|-----|------|--------|-------------|
| PaddleGrow | зелёный | ракетка ×1.5 | 10 сек |
| StickyPaddle | жёлтый | мяч прилипает | 10 сек |
| BallGrow | голубой | мяч ×1.5 | 10 сек |
| GunPaddle | оранжевый | пулемёт (Ctrl) | 15 сек |
| FireBall | красно-оранж. | пробивает блоки | 8 сек |
| MultiBall | фиолетовый | мяч → 3 мяча | мгновенно |

Падают вниз, активируются при касании ракеткой.
HUD показывает активные бонусы с оставшимися таймерами.

### 4.5 НЛО (UFO)
- Движутся горизонтально, отражаются от стен и блоков
- Уничтожаются за **2 удара** мячом или снарядами
- После уничтожения **респавнятся** случайно выше `y ∈ [200, 270]` или ниже `y ∈ [−120, 0]`

### 4.6 Бомбы (Bomb)
- Сбрасываются НЛО по таймеру, падают со скоростью −220 px/s
- Bomb ↔ Paddle → потеря жизни (или GameOver)
- Bomb ↔ Brick → бомба исчезает (блок не повреждается)

### 4.7 Снаряды пулемёта (Bullet)
- Ctrl (left/right), если активен GunPaddle; скорость +520 px/s
- Bullet ↔ Brick: 1 урон блоку, снаряд исчезает
- Bullet ↔ UFO: 1 урон НЛО, снаряд исчезает

---

## 5. Уровни

Данные — статический массив `LEVELS` в `src/resources/level_data.rs`.
Всего **5 уровней**, сетка `grid: &[&[u8]]` (0=пусто, 1=Normal, 2=Strong).
Также поддерживается **кастомный уровень** через редактор (`custom_level.lvl`).

| Уровень | Паттерн | НЛО | Скорость |
|---------|---------|-----|---------|
| 1 | Классический | 0 | ×1.0 |
| 2 | Шахматный | 1 | ×1.25 |
| 3 | Пирамида | 2 | ×1.5 |
| 4 | Крепость | 2 | ×1.75 |
| 5 | Хаос | 3 | ×2.0 |

---

## 6. Физика и коллизии

**AABB** (Axis-Aligned Bounding Box), FixedUpdate 64 Hz.

| Пара | Эффект |
|------|--------|
| Ball ↔ Wall | отскок |
| Ball ↔ Paddle | угловой отскок |
| Ball ↔ Brick | отскок + урон (FireBall — без отскока) |
| Ball ↔ UFO | отскок + урон НЛО |
| UFO ↔ Brick | разворот НЛО |
| Bonus ↔ Paddle | подбор бонуса |
| Bomb ↔ Paddle | потеря жизни |
| Bomb ↔ Brick | бомба исчезает |
| Bullet ↔ Brick | урон блоку, снаряд исчезает |
| Bullet ↔ UFO | урон НЛО, снаряд исчезает |

---

## 7. Управление

### Playing
| Клавиша | Действие |
|---------|----------|
| A / ← | ракетка влево |
| D / → | ракетка вправо |
| Пробел | запуск мяча |
| Ctrl (Left/Right) | стрельба пулемётом |
| Escape | пауза; повторно → MainMenu |
| F2 | музыка вкл/выкл |
| `*` Numpad | **[DEBUG]** следующий уровень |

### MainMenu
| Клавиша | Действие |
|---------|----------|
| W / ↑ или S / ↓ | навигация |
| Enter / Space | подтвердить выбор |

### LevelEditor
| Клавиша | Действие |
|---------|----------|
| ЛКМ / drag | рисовать кистью |
| ПКМ / drag | стирать |
| 0 / 1 / 2 | выбрать кисть |
| + / - | добавить/убрать ряд |
| S | сохранить в `custom_level.lvl` |
| L | загрузить из `custom_level.lvl` |
| P | играть кастомный уровень |
| Escape | в главное меню |

---

## 8. Состояния игры

```
Startup → MainMenu ──────────────── LevelEditor
               │                         │ ESC → MainMenu
               ↓ ENTER (Play Game)        │ P   → Playing
           Playing → LevelComplete → Playing
                  ↘ GameOver → Playing
```

| Состояние | Описание |
|-----------|----------|
| `MainMenu` | Меню: PLAY GAME / LEVEL EDITOR / QUIT |
| `Playing` | Активная игра |
| `LevelEditor` | Редактор кастомного уровня |
| `GameOver` | Игра окончена |
| `LevelComplete` | Уровень пройден |

**Пауза** — ресурс `Paused(bool)`, первый ESC → пауза, второй → MainMenu.

---

## 9. Архитектура (Bevy ECS)

### Плагины

| Плагин | Ответственность |
|--------|----------------|
| `AssetPlugin` | Загрузка ассетов, музыка, звуковые события |
| `GameplayPlugin` | Правила, ресурсы, победа/поражение, пауза |
| `PhysicsPlugin` | Ввод, движение, коллизии, бонусы, частицы |
| `UiPlugin` | HUD + экраны состояний |
| `LevelPlugin` | Камера, спавн/очистка уровня |
| `EditorPlugin` | Редактор уровней |

### Компоненты (только данные)

```
Velocity { x, y }
Collider { half_width, half_height }
Paddle { speed }
Ball { radius }  +  BallStuck (маркер)
Brick { brick_type, health, score_value }
Bonus { bonus_type }
PaddleGrowEffect / StickyEffect / BallGrowEffect / GunPaddleEffect / FireBallEffect
Bullet (маркер)
Particle { lifetime: Timer }
Ufo { speed, direction, bomb_timer, health }
Bomb { damage }
Wall (маркер)
LevelEntity (маркер для cleanup)
MusicController / MenuMusicController (маркеры музыки)
```

### Ресурсы

```
GameState (States enum): MainMenu | Playing | GameOver | LevelComplete | LevelEditor
Score, Lives, CurrentLevel, BallSpeedMultiplier
HighScore (сохранение в highscore.dat)
DebugSkipPending, Paused, MenuSelection
GameAssets (все Handle<AudioSource> и Handle<Image>)
MusicEnabled(bool)  — F2
EditorData          — данные редактора уровней
```

### Структура `src/`

```
src/
├── main.rs
├── app.rs
├── events.rs          SoundEvent (10 вариантов)
├── plugins/           asset, gameplay, physics, ui, level, editor
├── components/        ball, brick, bonus, bonus_effects, bomb, bullet,
│                      collider, level_entity, paddle, particle, ufo, velocity, wall
├── systems/           input, movement, collision, gameplay, bonus, gun,
│                      particles, ufo, editor
├── resources/         assets, game_state, score, level_data, editor
└── setup/             camera, level
```

---

## 10. UI

- **HUD**: SCORE (слева), LEVEL (центр), BEST (центр-право), LIVES (справа)
- **Бонусы**: строка под HUD, активные эффекты с оставшимся временем
- **Экраны**: MainMenu (3 пункта), GameOver, LevelComplete, Pause
- **Редактор**: сетка ячеек + UI подсказки (кисть, ряды, команды)
- Весь текст на **ASCII/латинице** — дефолтный шрифт Bevy не поддерживает кириллицу

---

## 11. Ассеты

```
assets/
├── music/    menu.ogg, gameplay.ogg
├── sounds/   ball_hit.ogg, brick_hit.ogg, brick_break.ogg, bonus_pickup.ogg,
│             life_lost.ogg, game_over.ogg, bullet_fire.ogg, ufo_hit.ogg, bomb_hit.ogg
└── sprites/  paddle.png, ball.png, ball_fire.png, brick_normal.png, brick_strong.png,
              brick_strong_hit.png, ufo.png, bullet.png, bomb.png, bonus_*.png ×6
```

Отсутствующие файлы не крашат игру — Bevy загружает асинхронно.

---

## 12. Требования к коду

- Чистая ECS-архитектура (Bevy)
- Компоненты — только данные, без логики
- Системы — только логика, без состояния
- Минимальная связанность через ECS (Query, Res, EventWriter/EventReader)
- Без сторонних физических библиотек
- При конфликте запросов Bevy (B0001) — `Without<T>` фильтры
