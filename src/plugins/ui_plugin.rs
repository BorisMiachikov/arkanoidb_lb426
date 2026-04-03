use bevy::app::AppExit;
use bevy::prelude::*;

use crate::components::bonus_effects::{BallGrowEffect, FireBallEffect, GunPaddleEffect, PaddleGrowEffect, StickyEffect};
use crate::resources::assets::GameAssets;
use crate::resources::editor::EditorData;
use crate::resources::game_state::GameState;
use crate::resources::score::{CurrentLevel, HighScore, Lives, MenuSelection, NameInput, OptionsSelection, Paused, Score, ScoreTable};
use crate::resources::settings::AppSettings;
use crate::setup::level::{WINDOW_WIDTH, WINDOW_HEIGHT};

// ─── Маркеры HUD ────────────────────────────────────────────────────────────

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct LevelText;

/// Маркер контейнера иконок жизней
#[derive(Component)]
struct LivesContainer;

/// Маркер одной иконки жизни
#[derive(Component)]
struct LivesIcon;

#[derive(Component)]
struct ActiveBonusText;

#[derive(Component)]
struct HighScoreText;

/// Маркер пункта главного меню (хранит индекс пункта)
#[derive(Component)]
struct MenuItemText(usize);

/// Маркер контейнера пункта меню (для обновления рамки при навигации)
#[derive(Component)]
struct MenuItemBox(usize);

/// Маркер пункта экрана Options (хранит индекс пункта)
#[derive(Component)]
struct OptionsItemText(usize);

/// Маркер строки на экране Options (для ховера/выделения)
#[derive(Component)]
struct OptionsRow(usize);

/// Маркер текста значения громкости (обновляется при изменении)
#[derive(Component)]
struct OptionsVolText(usize);

/// Маркер кнопки +/- громкости
#[derive(Component)]
struct OptionsVolBtn {
    vol_idx: usize,
    delta: i32,
}

/// Маркер строки ввода имени на экране EnterName
#[derive(Component)]
struct EnterNameText;

/// Маркер корневого узла HUD (для управления видимостью)
#[derive(Component)]
struct HudRoot;

// ─── Маркеры оверлеев ───────────────────────────────────────────────────────

/// Маркер: любой экран-оверлей (очищается при OnExit состояния)
#[derive(Component)]
struct OverlayScreen;

/// Маркер: оверлей паузы (управляется ресурсом Paused)
#[derive(Component)]
struct PauseOverlay;

/// Маркер фонового спрайта (меню / sat / редактор)
#[derive(Component)]
struct BackgroundSprite;

// ─── Плагин ─────────────────────────────────────────────────────────────────

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud);

        // Фоны
        app.add_systems(OnEnter(GameState::MainMenu),      spawn_menu_background);
        app.add_systems(OnEnter(GameState::Playing),       despawn_background);
        app.add_systems(OnEnter(GameState::LevelEditor),   (despawn_background, spawn_editor_background).chain());
        app.add_systems(OnExit(GameState::LevelEditor),    despawn_background);
        app.add_systems(OnEnter(GameState::GameOver),      spawn_sat_background);
        app.add_systems(OnExit(GameState::GameOver),       despawn_background);
        app.add_systems(OnEnter(GameState::LevelComplete), spawn_sat_background);
        app.add_systems(OnExit(GameState::LevelComplete),  despawn_background);

        // Главное меню
        app.add_systems(OnEnter(GameState::MainMenu), (reset_menu_selection, spawn_main_menu).chain());
        app.add_systems(OnExit(GameState::MainMenu), despawn_overlay);

        // Options
        app.add_systems(OnEnter(GameState::Options), (reset_options_selection, spawn_options_screen).chain());
        app.add_systems(OnExit(GameState::Options), despawn_overlay);

        // High Scores
        app.add_systems(OnEnter(GameState::HighScores), spawn_highscores_screen);
        app.add_systems(OnExit(GameState::HighScores), despawn_overlay);

        // Enter Name
        app.add_systems(OnEnter(GameState::EnterName), spawn_enter_name_screen);
        app.add_systems(OnExit(GameState::EnterName), despawn_overlay);

        // GameOver
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over);
        app.add_systems(OnExit(GameState::GameOver), despawn_overlay);

        // LevelComplete
        app.add_systems(OnEnter(GameState::LevelComplete), spawn_level_complete);
        app.add_systems(OnExit(GameState::LevelComplete), despawn_overlay);

        // Обновление HUD и паузы каждый кадр
        app.add_systems(
            Update,
            (
                update_hud_visibility,
                update_score_ui,
                update_lives_icons,
                update_level_ui,
                update_bonus_ui,
                update_highscore_ui,
                update_pause_overlay,
                update_menu_selection_ui.run_if(in_state(GameState::MainMenu)),
                menu_mouse_system.run_if(in_state(GameState::MainMenu)),
                update_options_ui.run_if(in_state(GameState::Options)),
                options_mouse_system.run_if(in_state(GameState::Options)),
                update_enter_name_ui.run_if(in_state(GameState::EnterName)),
            ),
        );
    }
}

// ─── HUD ────────────────────────────────────────────────────────────────────

fn update_hud_visibility(
    state: Res<State<GameState>>,
    mut query: Query<&mut Visibility, With<HudRoot>>,
) {
    if !state.is_changed() {
        return;
    }
    let visible = matches!(
        state.get(),
        GameState::Playing | GameState::LevelComplete | GameState::GameOver
    );
    if let Ok(mut vis) = query.single_mut() {
        *vis = if visible { Visibility::Visible } else { Visibility::Hidden };
    }
}

/// Хелпер: создаёт TextFont с пиксельным шрифтом
fn tf(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont { font: font.clone(), font_size: size, ..default() }
}

fn setup_hud(mut commands: Commands, assets: Res<GameAssets>) {
    let font = &assets.font_ui;
    commands
        .spawn((
        HudRoot,
        Visibility::Hidden,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..default()
        }))
        .with_children(|root| {
            // Верхняя строка: SCORE | LEVEL | LIVES
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("SCORE: 0"),
                    tf(font, 13.0),
                    TextColor(Color::WHITE),
                    ScoreText,
                ));
                row.spawn((
                    Text::new("LEVEL: 1"),
                    tf(font, 13.0),
                    TextColor(Color::srgb(0.9, 0.9, 0.3)),
                    LevelText,
                ));
                row.spawn((
                    Text::new("BEST: 0"),
                    tf(font, 13.0),
                    TextColor(Color::srgb(1.0, 0.8, 0.2)),
                    HighScoreText,
                ));
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        ..default()
                    },
                    LivesContainer,
                )).with_children(|lives_row| {
                    spawn_life_icon(lives_row);
                    spawn_life_icon(lives_row);
                    spawn_life_icon(lives_row);
                });
            });

            // Строка активных бонусов
            root.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(""),
                    tf(font, 10.0),
                    TextColor(Color::srgb(0.4, 1.0, 0.4)),
                    ActiveBonusText,
                ));
            });
        });
}

// ─── Обновление HUD ─────────────────────────────────────────────────────────

fn update_score_ui(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            **text = format!("SCORE: {}", score.value);
        }
    }
}

fn update_highscore_ui(
    highscore: Res<HighScore>,
    mut query: Query<&mut Text, With<HighScoreText>>,
) {
    if highscore.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            **text = format!("BEST: {}", highscore.value);
        }
    }
}

fn spawn_life_icon(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        LivesIcon,
        Node {
            width: Val::Px(28.0),
            height: Val::Px(8.0),
            border_radius: BorderRadius::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.6, 1.0)),
    ));
}

fn update_lives_icons(
    mut commands: Commands,
    lives: Res<Lives>,
    container_query: Query<Entity, With<LivesContainer>>,
) {
    if !lives.is_changed() {
        return;
    }
    let Ok(container) = container_query.single() else {
        return;
    };
    let count = lives.count;
    commands.entity(container)
        .despawn_children()
        .with_children(|parent| {
            for _ in 0..count {
                spawn_life_icon(parent);
            }
        });
}

fn update_level_ui(
    current_level: Res<CurrentLevel>,
    mut query: Query<&mut Text, With<LevelText>>,
) {
    if current_level.is_changed() {
        if let Ok(mut text) = query.single_mut() {
            **text = format!("LEVEL: {}", current_level.number + 1);
        }
    }
}

fn update_bonus_ui(
    mut query: Query<&mut Text, With<ActiveBonusText>>,
    paddle_grow: Query<&PaddleGrowEffect>,
    sticky: Query<&StickyEffect>,
    ball_grow: Query<&BallGrowEffect>,
    gun: Query<&GunPaddleEffect>,
    fire: Query<&FireBallEffect>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };

    let mut bonuses: Vec<String> = Vec::new();
    if let Ok(effect) = paddle_grow.single() {
        bonuses.push(format!("[PADDLE+ {:.1}s]", effect.timer.remaining_secs()));
    }
    if let Ok(effect) = sticky.single() {
        bonuses.push(format!("[STICKY {:.1}s]", effect.timer.remaining_secs()));
    }
    if let Ok(effect) = ball_grow.single() {
        bonuses.push(format!("[BALL+ {:.1}s]", effect.timer.remaining_secs()));
    }
    if let Ok(effect) = gun.single() {
        bonuses.push(format!("[GUN {:.1}s]", effect.timer.remaining_secs()));
    }
    if let Ok(effect) = fire.single() {
        bonuses.push(format!("[FIRE {:.1}s]", effect.timer.remaining_secs()));
    }

    **text = bonuses.join("  ");
}

// ─── Оверлеи состояний ──────────────────────────────────────────────────────

fn reset_menu_selection(mut selection: ResMut<MenuSelection>) {
    selection.0 = 0;
}

fn despawn_overlay(mut commands: Commands, query: Query<Entity, With<OverlayScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_overlay_root(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            OverlayScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id()
}

fn spawn_panel(parent: &mut ChildSpawnerCommands, children: impl FnOnce(&mut ChildSpawnerCommands)) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(48.0)),
                row_gap: Val::Px(20.0),
                border_radius: BorderRadius::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.08, 0.92)),
        ))
        .with_children(children);
}

fn spawn_title(parent: &mut ChildSpawnerCommands, text: &str, color: Color, font: &Handle<Font>) {
    parent.spawn((
        Text::new(text),
        tf(font, 28.0),
        TextColor(color),
    ));
}

fn spawn_subtitle(parent: &mut ChildSpawnerCommands, text: &str, font: &Handle<Font>) {
    parent.spawn((
        Text::new(text),
        tf(font, 14.0),
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
    ));
}

fn spawn_hint(parent: &mut ChildSpawnerCommands, text: &str, font: &Handle<Font>) {
    parent.spawn((
        Text::new(text),
        tf(font, 9.0),
        TextColor(Color::srgb(0.55, 0.55, 0.55)),
    ));
}

const MENU_ITEMS: &[&str] = &["PLAY GAME", "LEVEL EDITOR", "HIGH SCORES", "OPTIONS", "QUIT"];

// Цвета декоративных кирпичей: border и тёмный фон
const DECO_BORDER: [Color; 6] = [
    Color::srgb(0.2, 0.75, 1.0),
    Color::srgb(0.2, 0.85, 0.3),
    Color::srgb(0.95, 0.85, 0.1),
    Color::srgb(0.95, 0.52, 0.1),
    Color::srgb(0.85, 0.2, 0.9),
    Color::srgb(0.95, 0.2, 0.2),
];
const DECO_BG: [Color; 6] = [
    Color::srgba(0.2, 0.75, 1.0,  0.18),
    Color::srgba(0.2, 0.85, 0.3,  0.18),
    Color::srgba(0.95, 0.85, 0.1, 0.18),
    Color::srgba(0.95, 0.52, 0.1, 0.18),
    Color::srgba(0.85, 0.2, 0.9,  0.18),
    Color::srgba(0.95, 0.2, 0.2,  0.18),
];

// Ряды декоративных кирпичей (индексы 0–5 в DECO_BORDER/BG)
const LEFT_BRICKS:  &[&[usize]] = &[&[0,2,4], &[1,3,5], &[4,0,2]];
const RIGHT_BRICKS: &[&[usize]] = &[&[2,5,3], &[1,0,4], &[5,3,1]];

fn spawn_deco_column(
    parent: &mut ChildSpawnerCommands,
    groups: &[&[usize]],
    align: AlignItems,
) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Column,
        align_items: align,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(10.0),
        min_width: Val::Px(100.0),
        height: Val::Percent(100.0),
        ..default()
    }).with_children(|col| {
        for &group in groups {
            col.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                ..default()
            }).with_children(|g| {
                for &ci in group {
                    g.spawn((
                        Node {
                            width: Val::Px(84.0),
                            height: Val::Px(20.0),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BorderColor::all(DECO_BORDER[ci]),
                        BackgroundColor(DECO_BG[ci]),
                    ));
                }
            });
        }
    });
}

// Главное меню
fn spawn_main_menu(mut commands: Commands, highscore: Res<HighScore>, assets: Res<GameAssets>) {
    let font = &assets.font_ui;
    let best = highscore.value;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|screen| {
        // Горизонтальная строка: [левые кирпичи | центр | правые кирпичи]
        screen.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        }).with_children(|row| {

            // Центральная колонка
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::horizontal(Val::Px(32.0)),
                min_width: Val::Px(340.0),
                ..default()
            }).with_children(|center| {
                // Заголовок
                center.spawn((
                    Text::new("ARKANOID"),
                    tf(font, 36.0),
                    TextColor(Color::srgb(0.15, 0.9, 1.0)),
                ));

                // Рекорд
                let best_text = if best > 0 {
                    format!("BEST: {}", best)
                } else {
                    "BEST: ---".to_string()
                };
                center.spawn((
                    Text::new(best_text),
                    tf(font, 12.0),
                    TextColor(Color::srgb(0.35, 0.85, 1.0)),
                ));

                // Разделитель
                center.spawn(Node { height: Val::Px(8.0), ..default() });

                // Пункты меню
                for (idx, &label) in MENU_ITEMS.iter().enumerate() {
                    let selected = idx == 0;
                    center.spawn((
                        MenuItemBox(idx),
                        Interaction::default(),
                        Node {
                            width: Val::Px(300.0),
                            padding: UiRect::axes(Val::Px(18.0), Val::Px(9.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: if selected {
                                UiRect::all(Val::Px(2.0))
                            } else {
                                UiRect::ZERO
                            },
                            ..default()
                        },
                        BorderColor::all(if selected {
                            Color::srgb(0.75, 0.25, 1.0)
                        } else {
                            Color::srgba(0.0, 0.0, 0.0, 0.0)
                        }),
                        BackgroundColor(if selected {
                            Color::srgba(0.45, 0.1, 0.65, 0.28)
                        } else {
                            Color::srgba(0.0, 0.0, 0.0, 0.0)
                        }),
                    )).with_children(|item| {
                        let prefix = if selected { "> " } else { "  " };
                        item.spawn((
                            Text::new(format!("{}{}", prefix, label)),
                            tf(font, 16.0),
                            TextColor(if selected {
                                Color::WHITE
                            } else {
                                Color::srgb(0.52, 0.52, 0.52)
                            }),
                            MenuItemText(idx),
                        ));
                    });
                }

            });

        });
    });
}

fn update_menu_selection_ui(
    selection: Res<MenuSelection>,
    mut text_q: Query<(&mut Text, &mut TextColor, &MenuItemText)>,
    mut box_q: Query<(&mut Node, &mut BorderColor, &mut BackgroundColor, &MenuItemBox)>,
) {
    if !selection.is_changed() {
        return;
    }
    for (mut text, mut color, item) in &mut text_q {
        let selected = item.0 == selection.0;
        let prefix = if selected { "> " } else { "  " };
        **text = format!("{}{}", prefix, MENU_ITEMS[item.0]);
        color.0 = if selected { Color::WHITE } else { Color::srgb(0.52, 0.52, 0.52) };
    }
    for (mut node, mut border, mut bg, item) in &mut box_q {
        let selected = item.0 == selection.0;
        node.border = if selected { UiRect::all(Val::Px(2.0)) } else { UiRect::ZERO };
        border.set_all(if selected {
            Color::srgb(0.75, 0.25, 1.0)
        } else {
            Color::srgba(0.0, 0.0, 0.0, 0.0)
        });
        bg.0 = if selected {
            Color::srgba(0.45, 0.1, 0.65, 0.28)
        } else {
            Color::srgba(0.0, 0.0, 0.0, 0.0)
        };
    }
}

fn menu_mouse_system(
    mut selection: ResMut<MenuSelection>,
    mut next_state: ResMut<NextState<GameState>>,
    mut editor: ResMut<EditorData>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut current_level: ResMut<CurrentLevel>,
    mut app_exit: MessageWriter<AppExit>,
    query: Query<(&Interaction, &MenuItemBox), Changed<Interaction>>,
) {
    for (interaction, item) in &query {
        match interaction {
            Interaction::Hovered => {
                selection.0 = item.0;
            }
            Interaction::Pressed => {
                match item.0 {
                    0 => {
                        score.value = 0;
                        lives.count = 3;
                        current_level.number = 0;
                        editor.active = false;
                        next_state.set(GameState::Playing);
                    }
                    1 => next_state.set(GameState::LevelEditor),
                    2 => next_state.set(GameState::HighScores),
                    3 => next_state.set(GameState::Options),
                    4 => { app_exit.write(AppExit::Success); }
                    _ => {}
                }
            }
            Interaction::None => {}
        }
    }
}

// ─── Options ────────────────────────────────────────────────────────────────

fn reset_options_selection(mut selection: ResMut<OptionsSelection>) {
    selection.0 = 0;
}

fn vol_pct(v: f32) -> u32 { (v * 100.0).round() as u32 }

fn spawn_vol_btn(parent: &mut ChildSpawnerCommands, vol_idx: usize, delta: i32, font: &Handle<Font>) {
    parent.spawn((
        OptionsVolBtn { vol_idx, delta },
        Interaction::default(),
        Node {
            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.5)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::srgb(0.4, 0.5, 0.9)),
        BackgroundColor(Color::srgba(0.2, 0.2, 0.5, 0.35)),
    )).with_children(|b| {
        b.spawn((
            Text::new(if delta < 0 { "-" } else { "+" }),
            tf(font, 16.0),
            TextColor(Color::srgb(0.8, 0.85, 1.0)),
        ));
    });
}

fn spawn_options_screen(
    mut commands: Commands,
    selection: Res<OptionsSelection>,
    settings: Res<AppSettings>,
    assets: Res<GameAssets>,
) {
    let font = &assets.font_ui;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "OPTIONS", Color::srgb(0.3, 0.8, 1.0), font);

            // Строки громкости
            let labels = ["MUSIC VOLUME", "SFX VOLUME"];
            let vols   = [settings.music_volume, settings.sfx_volume];
            for idx in 0..2usize {
                let selected = selection.0 == idx;
                panel.spawn((
                    OptionsRow(idx),
                    Interaction::default(),
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                        width: Val::Px(400.0),
                        border: if selected { UiRect::all(Val::Px(1.5)) } else { UiRect::ZERO },
                        ..default()
                    },
                    BorderColor::all(if selected {
                        Color::srgb(0.75, 0.25, 1.0)
                    } else {
                        Color::srgba(0.0, 0.0, 0.0, 0.0)
                    }),
                    BackgroundColor(if selected {
                        Color::srgba(0.45, 0.1, 0.65, 0.2)
                    } else {
                        Color::srgba(0.0, 0.0, 0.0, 0.0)
                    }),
                )).with_children(|row| {
                    // Метка
                    row.spawn((
                        Text::new(labels[idx]),
                        tf(font, 13.0),
                        TextColor(Color::srgb(0.75, 0.75, 0.75)),
                        Node { flex_grow: 1.0, ..default() },
                    ));
                    // Кнопка −
                    spawn_vol_btn(row, idx, -1, font);
                    // Значение %
                    row.spawn((
                        OptionsVolText(idx),
                        Text::new(format!("{:3}%", vol_pct(vols[idx]))),
                        tf(font, 13.0),
                        TextColor(Color::WHITE),
                        Node {
                            min_width: Val::Px(48.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    ));
                    // Кнопка +
                    spawn_vol_btn(row, idx, 1, font);
                });
            }

            // Строка BACK
            let selected = selection.0 == 2;
            panel.spawn((
                OptionsRow(2),
                Interaction::default(),
                Node {
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    width: Val::Px(400.0),
                    border: if selected { UiRect::all(Val::Px(1.5)) } else { UiRect::ZERO },
                    ..default()
                },
                BorderColor::all(if selected {
                    Color::srgb(0.75, 0.25, 1.0)
                } else {
                    Color::srgba(0.0, 0.0, 0.0, 0.0)
                }),
                BackgroundColor(if selected {
                    Color::srgba(0.45, 0.1, 0.65, 0.2)
                } else {
                    Color::srgba(0.0, 0.0, 0.0, 0.0)
                }),
            )).with_children(|row| {
                let prefix = if selected { "> " } else { "  " };
                row.spawn((
                    Text::new(format!("{}BACK", prefix)),
                    tf(font, 16.0),
                    TextColor(if selected { Color::WHITE } else { Color::srgb(0.55, 0.55, 0.55) }),
                    OptionsItemText(2),
                ));
            });

            spawn_hint(panel, "[ W/S  Navigate ]  [ LEFT/RIGHT  Adjust ]  [ ESC  Back ]", font);
        });
    });
}

fn update_options_ui(
    selection: Res<OptionsSelection>,
    settings: Res<AppSettings>,
    mut vol_q: Query<(&mut Text, &OptionsVolText), Without<OptionsItemText>>,
    mut back_q: Query<(&mut Text, &mut TextColor, &OptionsItemText), Without<OptionsVolText>>,
    mut row_q: Query<(&mut Node, &mut BorderColor, &mut BackgroundColor, &OptionsRow)>,
) {
    if !selection.is_changed() && !settings.is_changed() {
        return;
    }
    // Обновить значения %
    let vols = [settings.music_volume, settings.sfx_volume];
    for (mut text, vol) in &mut vol_q {
        **text = format!("{:3}%", vol_pct(vols[vol.0]));
    }
    // Обновить BACK
    if let Ok((mut text, mut color, _)) = back_q.single_mut() {
        let sel = selection.0 == 2;
        **text = format!("{}BACK", if sel { "> " } else { "  " });
        color.0 = if sel { Color::WHITE } else { Color::srgb(0.55, 0.55, 0.55) };
    }
    // Обновить подсветку строк
    for (mut node, mut border, mut bg, row) in &mut row_q {
        let sel = row.0 == selection.0;
        node.border = if sel { UiRect::all(Val::Px(1.5)) } else { UiRect::ZERO };
        border.set_all(if sel { Color::srgb(0.75, 0.25, 1.0) } else { Color::srgba(0.0, 0.0, 0.0, 0.0) });
        bg.0 = if sel { Color::srgba(0.45, 0.1, 0.65, 0.2) } else { Color::srgba(0.0, 0.0, 0.0, 0.0) };
    }
}

fn options_mouse_system(
    mut selection: ResMut<OptionsSelection>,
    mut settings: ResMut<AppSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    btn_q: Query<(&Interaction, &OptionsVolBtn), Changed<Interaction>>,
    row_q: Query<(&Interaction, &OptionsRow), Changed<Interaction>>,
) {
    // Клики по кнопкам +/-
    for (interaction, btn) in &btn_q {
        if *interaction == Interaction::Pressed {
            let vol = if btn.vol_idx == 0 {
                &mut settings.music_volume
            } else {
                &mut settings.sfx_volume
            };
            *vol = (*vol + btn.delta as f32 * 0.1).clamp(0.0, 1.0);
        }
    }
    // Ховер/клик по строкам
    for (interaction, row) in &row_q {
        if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            selection.0 = row.0;
        }
        if *interaction == Interaction::Pressed && row.0 == 2 {
            next_state.set(GameState::MainMenu);
        }
    }
}

// ─── High Scores ────────────────────────────────────────────────────────────

fn spawn_highscores_screen(mut commands: Commands, score_table: Res<ScoreTable>, assets: Res<GameAssets>) {
    let font = &assets.font_ui;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "HIGH SCORES", Color::srgb(1.0, 0.8, 0.2), font);

            if score_table.entries.is_empty() {
                spawn_subtitle(panel, "No scores yet. Be the first!", font);
            } else {
                for (i, entry) in score_table.entries.iter().enumerate() {
                    let color = match i {
                        0 => Color::srgb(1.0, 0.85, 0.1),
                        1 => Color::srgb(0.85, 0.85, 0.85),
                        2 => Color::srgb(0.9, 0.6, 0.3),
                        _ => Color::srgb(0.65, 0.65, 0.65),
                    };
                    let text = format!("{:2}.  {:<12}{:>7}", i + 1, entry.name, entry.score);
                    panel.spawn((
                        Text::new(text),
                        tf(font, 12.0),
                        TextColor(color),
                    ));
                }
            }

            spawn_hint(panel, "[ ENTER / ESC  Back ]", font);
        });
    });
}

// ─── Enter Name ─────────────────────────────────────────────────────────────

fn spawn_enter_name_screen(
    mut commands: Commands,
    score: Res<Score>,
    name_input: Res<NameInput>,
    assets: Res<GameAssets>,
) {
    let font = &assets.font_ui;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "NEW HIGH SCORE!", Color::srgb(1.0, 0.9, 0.2), font);
            spawn_subtitle(panel, &format!("Score: {}", score.value), font);

            panel.spawn((
                Text::new(format!("> {}_", name_input.text)),
                tf(font, 18.0),
                TextColor(Color::WHITE),
                EnterNameText,
            ));

            spawn_hint(panel, "[ Letters & digits - max 10 chars ]", font);
            spawn_hint(panel, "[ ENTER  Save ]  [ ESC  Skip ]", font);
        });
    });
}

fn update_enter_name_ui(
    name_input: Res<NameInput>,
    mut query: Query<&mut Text, With<EnterNameText>>,
) {
    if !name_input.is_changed() { return; }
    if let Ok(mut text) = query.single_mut() {
        **text = format!("> {}_", name_input.text);
    }
}

// GameOver
fn spawn_game_over(
    mut commands: Commands,
    score: Res<Score>,
    highscore: Res<HighScore>,
    score_table: Res<ScoreTable>,
    assets: Res<GameAssets>,
) {
    let font = &assets.font_ui;
    let score_val = score.value;
    let best = highscore.value;
    let is_new_record = score_val > 0 && score_val >= best;
    let qualifies = score_table.qualifies(score_val);
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "GAME OVER", Color::srgb(1.0, 0.25, 0.25), font);
            spawn_subtitle(panel, &format!("Score: {}", score_val), font);
            if is_new_record {
                spawn_subtitle(panel, "*** NEW RECORD! ***", font);
            } else if best > 0 {
                spawn_hint(panel, &format!("Best: {}", best), font);
            }
            if qualifies {
                spawn_hint(panel, "[ ENTER - Add to High Scores ]  [ ESC - Menu ]", font);
            } else {
                spawn_hint(panel, "[ ENTER - Restart ]  [ ESC - Menu ]", font);
            }
        });
    });
}

// LevelComplete
fn spawn_level_complete(
    mut commands: Commands,
    score: Res<Score>,
    current_level: Res<CurrentLevel>,
    assets: Res<GameAssets>,
) {
    let font = &assets.font_ui;
    let score_val = score.value;
    let level_num = current_level.number + 1;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "LEVEL COMPLETE!", Color::srgb(0.3, 1.0, 0.4), font);
            spawn_subtitle(panel, &format!("Level {}  |  Score: {}", level_num, score_val), font);
            spawn_hint(panel, "[ ENTER - Next Level ]", font);
        });
    });
}

// ─── Оверлей паузы ──────────────────────────────────────────────────────────

fn update_pause_overlay(
    mut commands: Commands,
    paused: Res<Paused>,
    overlay_query: Query<Entity, With<PauseOverlay>>,
    state: Res<State<GameState>>,
    assets: Res<GameAssets>,
) {
    let font = &assets.font_ui;
    if !paused.is_changed() {
        return;
    }

    // Показываем паузу только в Playing
    if paused.0 && *state.get() == GameState::Playing {
        commands
            .spawn((
                PauseOverlay,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|parent| {
                spawn_panel(parent, |panel| {
                    spawn_title(panel, "PAUSE", Color::srgb(0.9, 0.9, 0.3), font);
                    spawn_hint(panel, "[ ESC  Resume ]  [ ESC x2  Main Menu ]", font);
                });
            });
    } else {
        for entity in &overlay_query {
            commands.entity(entity).despawn();
        }
    }
}

// ─── Фоны ───────────────────────────────────────────────────────────────────

fn spawn_background_sprite(commands: &mut Commands, handle: Handle<Image>) {
    commands.spawn((
        BackgroundSprite,
        Sprite {
            image: handle,
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
}

fn spawn_menu_background(
    mut commands: Commands,
    assets: Res<GameAssets>,
    query: Query<(), With<BackgroundSprite>>,
) {
    if !query.is_empty() {
        return;
    }
    spawn_background_sprite(&mut commands, assets.bg_menu.clone());
}

fn spawn_sat_background(mut commands: Commands, assets: Res<GameAssets>) {
    spawn_background_sprite(&mut commands, assets.bg_game_sat.clone());
}

fn spawn_editor_background(mut commands: Commands, assets: Res<GameAssets>) {
    spawn_background_sprite(&mut commands, assets.bg_editor.clone());
}

fn despawn_background(mut commands: Commands, query: Query<Entity, With<BackgroundSprite>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
