# Техническое задание: Игра в жанре Arkanoid/Breakout (Rust + Bevy 0.18)

## 1. Общая информация

**Название проекта:** arkanoidb_lb426
**Жанр:** Аркада / Breakout
**Язык разработки:** Rust (stable)
**Движок:** Bevy 0.18
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
| ExtraLife | розовый | +1 жизнь (макс. 9) | мгновенно |

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
Всего **5 встроенных уровней**, сетка `grid: &[&[u8]]`.

**Кодировка ячеек:**
- `0` = пусто
- `1–6` = Normal (цвета: Blue/Green/Yellow/Orange/Purple/Red)
- `7–12` = Strong (те же 6 цветов)

Цвет блока **не зависит от номера ряда** — задаётся явно кодом ячейки.

Пользовательские уровни сохраняются в `levels/level_N.lvl` и при старте уровня имеют **приоритет** над встроенными данными. Новые уровни (6, 7, 8...) создаются в редакторе клавишей `N`.

| Уровень | Паттерн | НЛО | Скорость |
|---------|---------|-----|---------|
| 1 | Классический | 0 | ×1.0 |
| 2 | Шахматный | 1 | ×1.25 |
| 3 | Пирамида | 2 | ×1.5 |
| 4 | Крепость | 3 | ×1.75 |
| 5 | Финальная стена | 4 | ×2.0 |

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

### MainMenu / Options
| Клавиша | Действие |
|---------|----------|
| W / ↑ или S / ↓ | навигация |
| Enter / Space | подтвердить выбор |
| ← / → | изменить громкость (только в Options) |
| ESC | назад в MainMenu (только в Options) |

### HighScores
| Клавиша | Действие |
|---------|----------|
| Enter / ESC | вернуться в MainMenu |

### EnterName
| Клавиша | Действие |
|---------|----------|
| Буквы / цифры | ввод имени (макс. 12 символов) |
| Backspace | удалить последний символ |
| Enter | подтвердить имя и сохранить рекорд |

### GameOver
| Клавиша | Действие |
|---------|----------|
| Enter / Space | рестарт с первого уровня |
| ESC | выход в MainMenu |

### LevelEditor
| Клавиша | Действие |
|---------|----------|
| ЛКМ / drag | рисовать кистью |
| ПКМ / drag | стирать |
| 1–6 | выбрать цвет кисти |
| T | переключить тип Normal ↔ Strong |
| 0 | кисть-стёрка |
| ← / → | предыдущий / следующий уровень |
| N | создать новый пустой уровень |
| + / - | добавить/убрать ряд |
| S | сохранить в `levels/level_N.lvl` |
| L | загрузить из `levels/level_N.lvl` |
| P | играть редактируемый уровень |
| Escape | в главное меню |

---

## 8. Состояния игры

```
Startup → MainMenu ── Options    (ESC → MainMenu)
               │    ├ HighScores (ESC/Enter → MainMenu)
               │    ├ EnterName  (Enter → HighScores)
               │    └ LevelEditor (ESC → MainMenu, P → Playing)
               ↓ ENTER (Play Game)
           Playing → LevelComplete → Playing
                  ↘ GameOver → EnterName (если рекорд) → HighScores → MainMenu
                            ↘ Playing (Enter, без рекорда)
                            ↘ MainMenu (ESC)
```

| Состояние | Описание |
|-----------|----------|
| `MainMenu` | Меню: PLAY GAME / LEVEL EDITOR / HIGH SCORES / OPTIONS / QUIT |
| `Options` | Настройки громкости музыки и SFX |
| `HighScores` | Таблица рекордов топ-10 (из `scores.dat`) |
| `EnterName` | Ввод имени при новом рекорде |
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
GameState (States enum): MainMenu | Options | HighScores | EnterName | Playing | GameOver | LevelComplete | LevelEditor
Score, Lives, CurrentLevel, BallSpeedMultiplier
HighScore (сохранение в highscore.dat)
ScoreTable { entries: Vec<ScoreEntry> }  — топ-10, сохранение в scores.dat
NameInput { text: String }               — буферный ввод имени в EnterName
DebugSkipPending, Paused, MenuSelection, OptionsSelection
AudioSettings { music_volume, sfx_volume }  — управляется в Options
GameAssets (все Handle<AudioSource>, Handle<Image> ×15, Handle<Font> font_ui)
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

- **HUD**: SCORE (слева), LEVEL (центр), BEST (центр-право), LIVES — иконки-ракетки (справа). Виден **только** в состояниях Playing / LevelComplete / GameOver.
- **Бонусы**: строка под HUD, активные эффекты с оставшимся временем
- **Экраны**: MainMenu (5 пунктов), Options (громкость), HighScores (топ-10), EnterName (ввод имени), GameOver, LevelComplete, Pause
- **Редактор**: сетка 12 рядов, выбор цвета (1–6) и типа (T), навигация по уровням (←/→), создание нового (N)
- Весь текст на **ASCII/латинице** — дефолтный шрифт Bevy не поддерживает кириллицу
- UI-изображения: `ImageNode` (не `Sprite`)

---

## 11. Ассеты

```
assets/
├── fonts/    PressStart2P-Regular.ttf  ← пиксельный шрифт UI (GameAssets.font_ui)
├── music/    menu.ogg, gameplay.ogg
├── sounds/   ball_hit.ogg, brick_hit.ogg, brick_break.ogg, bonus_pickup.ogg,
│             life_lost.ogg, game_over.ogg, bullet_fire.ogg, ufo_hit.ogg, bomb_hit.ogg
└── sprites/  padle.png (⚠ опечатка в имени — именно так), ball.png, ball_fire.png,
              brick_normal.png, brick_strong.png, brick_strong_hit.png,
              ufo.png, bullet.png, bomb.png, bonus_*.png ×6
```

Спецификация спрайтов (размеры, форматы): `docs/sprites_spec.md`.

Отсутствующие файлы не крашат игру — Bevy загружает асинхронно.
Ракетка и блоки используют `Sprite`; остальные объекты — `Mesh2d`.

---

## 12. Технические детали

- **Камера**: `ScalingMode::AutoMin { min_width: 800.0, min_height: 600.0 }` — виртуальное пространство 800×600, масштабируется под размер окна с letterbox/pillarbox. Окно `resizable: true`.
- **Release-сборка**: `opt-level="z"`, `lto=true`, `codegen-units=1`, `strip=true` — 54 МБ → 26 МБ.
- **Физика**: `FixedUpdate` 64 Hz, `dt.min(0.05)` против туннелирования.
- **Events**: в Bevy 0.18 `EventWriter`/`EventReader` заменены на `MessageWriter`/`MessageReader`; тип деривируется через `#[derive(Message)]`; регистрация через `app.add_message::<T>()`.

---

## 13. Требования к коду

- Чистая ECS-архитектура (Bevy)
- Компоненты — только данные, без логики
- Системы — только логика, без состояния
- Минимальная связанность через ECS (Query, Res, EventWriter/EventReader)
- Без сторонних физических библиотек
- При конфликте запросов Bevy (B0001) — `Without<T>` фильтры
