use bevy::prelude::*;

use crate::components::bonus_effects::{BallGrowEffect, FireBallEffect, GunPaddleEffect, PaddleGrowEffect, StickyEffect};
use crate::resources::game_state::GameState;
use crate::resources::score::{AudioSettings, CurrentLevel, HighScore, Lives, MenuSelection, OptionsSelection, Paused, Score};

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

/// Маркер пункта экрана Options (хранит индекс пункта)
#[derive(Component)]
struct OptionsItemText(usize);

// ─── Маркеры оверлеев ───────────────────────────────────────────────────────

/// Маркер: любой экран-оверлей (очищается при OnExit состояния)
#[derive(Component)]
struct OverlayScreen;

/// Маркер: оверлей паузы (управляется ресурсом Paused)
#[derive(Component)]
struct PauseOverlay;

// ─── Плагин ─────────────────────────────────────────────────────────────────

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud);

        // Главное меню
        app.add_systems(OnEnter(GameState::MainMenu), (reset_menu_selection, spawn_main_menu).chain());
        app.add_systems(OnExit(GameState::MainMenu), despawn_overlay);

        // Options
        app.add_systems(OnEnter(GameState::Options), (reset_options_selection, spawn_options_screen).chain());
        app.add_systems(OnExit(GameState::Options), despawn_overlay);

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
                update_score_ui,
                update_lives_icons,
                update_level_ui,
                update_bonus_ui,
                update_highscore_ui,
                update_pause_overlay,
                update_menu_selection_ui.run_if(in_state(GameState::MainMenu)),
                update_options_ui.run_if(in_state(GameState::Options)),
            ),
        );
    }
}

// ─── HUD ────────────────────────────────────────────────────────────────────

fn setup_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|root| {
            // Верхняя строка: SCORE | LEVEL | LIVES
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("SCORE: 0"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                    ScoreText,
                ));
                row.spawn((
                    Text::new("LEVEL: 1"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.3)),
                    LevelText,
                ));
                row.spawn((
                    Text::new("BEST: 0"),
                    TextFont { font_size: 18.0, ..default() },
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
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.4, 1.0, 0.4)),
                    ActiveBonusText,
                ));
            });
        });
}

// ─── Обновление HUD ─────────────────────────────────────────────────────────

fn update_score_ui(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            **text = format!("SCORE: {}", score.value);
        }
    }
}

fn update_highscore_ui(
    highscore: Res<HighScore>,
    mut query: Query<&mut Text, With<HighScoreText>>,
) {
    if highscore.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            **text = format!("BEST: {}", highscore.value);
        }
    }
}

fn spawn_life_icon(parent: &mut ChildBuilder) {
    parent.spawn((
        LivesIcon,
        Node {
            width: Val::Px(28.0),
            height: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.6, 1.0)),
        BorderRadius::all(Val::Px(2.0)),
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
    let Ok(container) = container_query.get_single() else {
        return;
    };
    let count = lives.count;
    commands.entity(container)
        .despawn_descendants()
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
        if let Ok(mut text) = query.get_single_mut() {
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
    let Ok(mut text) = query.get_single_mut() else {
        return;
    };

    let mut bonuses: Vec<String> = Vec::new();
    if let Ok(effect) = paddle_grow.get_single() {
        bonuses.push(format!("[PADDLE+ {:.1}s]", effect.timer.remaining_secs()));
    }
    if let Ok(effect) = sticky.get_single() {
        bonuses.push(format!("[STICKY {:.1}s]", effect.timer.remaining_secs()));
    }
    if let Ok(effect) = ball_grow.get_single() {
        bonuses.push(format!("[BALL+ {:.1}s]", effect.timer.remaining_secs()));
    }
    if let Ok(effect) = gun.get_single() {
        bonuses.push(format!("[GUN {:.1}s]", effect.timer.remaining_secs()));
    }
    if let Ok(effect) = fire.get_single() {
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
        commands.entity(entity).despawn_recursive();
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

fn spawn_panel(parent: &mut ChildBuilder, children: impl FnOnce(&mut ChildBuilder)) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(48.0)),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.08, 0.92)),
            BorderRadius::all(Val::Px(16.0)),
        ))
        .with_children(children);
}

fn spawn_title(parent: &mut ChildBuilder, text: &str, color: Color) {
    parent.spawn((
        Text::new(text),
        TextFont { font_size: 52.0, ..default() },
        TextColor(color),
    ));
}

fn spawn_subtitle(parent: &mut ChildBuilder, text: &str) {
    parent.spawn((
        Text::new(text),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
    ));
}

fn spawn_hint(parent: &mut ChildBuilder, text: &str) {
    parent.spawn((
        Text::new(text),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.55, 0.55, 0.55)),
    ));
}

const MENU_ITEMS: &[&str] = &["PLAY GAME", "LEVEL EDITOR", "OPTIONS", "QUIT"];

// Главное меню
fn spawn_main_menu(mut commands: Commands, highscore: Res<HighScore>) {
    let best = highscore.value;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "ARKANOID", Color::srgb(0.3, 0.7, 1.0));
            if best > 0 {
                spawn_subtitle(panel, &format!("Best Score: {}", best));
            }

            // Пункты меню
            for (idx, &label) in MENU_ITEMS.iter().enumerate() {
                let prefix = if idx == 0 { ">  " } else { "   " };
                panel.spawn((
                    Text::new(format!("{}{}", prefix, label)),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(if idx == 0 {
                        Color::WHITE
                    } else {
                        Color::srgb(0.55, 0.55, 0.55)
                    }),
                    MenuItemText(idx),
                ));
            }

            spawn_hint(panel, "[ W/S or UP/DN  Select ]  [ ENTER  Confirm ]");
        });
    });
}

fn update_menu_selection_ui(
    selection: Res<MenuSelection>,
    mut query: Query<(&mut Text, &mut TextColor, &MenuItemText)>,
) {
    if !selection.is_changed() {
        return;
    }
    for (mut text, mut color, item) in &mut query {
        let selected = item.0 == selection.0;
        let prefix = if selected { ">  " } else { "   " };
        let label = MENU_ITEMS[item.0];
        **text = format!("{}{}", prefix, label);
        color.0 = if selected {
            Color::WHITE
        } else {
            Color::srgb(0.55, 0.55, 0.55)
        };
    }
}

// ─── Options ────────────────────────────────────────────────────────────────

fn reset_options_selection(mut selection: ResMut<OptionsSelection>) {
    selection.0 = 0;
}

fn options_item_text(idx: usize, selected: usize, settings: &AudioSettings) -> (String, Color) {
    let prefix = if idx == selected { ">  " } else { "   " };
    let label = match idx {
        0 => format!("MUSIC VOLUME:   {}%", (settings.music_volume * 100.0).round() as u32),
        1 => format!("SFX VOLUME:     {}%", (settings.sfx_volume   * 100.0).round() as u32),
        _ => "BACK".to_string(),
    };
    let color = if idx == selected { Color::WHITE } else { Color::srgb(0.55, 0.55, 0.55) };
    (format!("{}{}", prefix, label), color)
}

fn spawn_options_screen(
    mut commands: Commands,
    selection: Res<OptionsSelection>,
    settings: Res<AudioSettings>,
) {
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "OPTIONS", Color::srgb(0.3, 0.8, 1.0));

            for idx in 0..3usize {
                let (text, color) = options_item_text(idx, selection.0, &settings);
                panel.spawn((
                    Text::new(text),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(color),
                    OptionsItemText(idx),
                ));
            }

            spawn_hint(panel, "[ W/S  Navigate ]  [ LEFT/RIGHT  Adjust ]  [ ESC  Back ]");
        });
    });
}

fn update_options_ui(
    selection: Res<OptionsSelection>,
    settings: Res<AudioSettings>,
    mut query: Query<(&mut Text, &mut TextColor, &OptionsItemText)>,
) {
    if !selection.is_changed() && !settings.is_changed() {
        return;
    }
    for (mut text, mut color, item) in &mut query {
        let (t, c) = options_item_text(item.0, selection.0, &settings);
        **text = t;
        color.0 = c;
    }
}

// GameOver
fn spawn_game_over(mut commands: Commands, score: Res<Score>, highscore: Res<HighScore>) {
    let score_val = score.value;
    let best = highscore.value;
    let is_new_record = score_val > 0 && score_val >= best;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "GAME OVER", Color::srgb(1.0, 0.25, 0.25));
            spawn_subtitle(panel, &format!("Score: {}", score_val));
            if is_new_record {
                spawn_subtitle(panel, "*** NEW RECORD! ***");
            } else {
                spawn_hint(panel, &format!("Best: {}", best));
            }
            spawn_hint(panel, "[ ENTER to Restart ]");
        });
    });
}

// LevelComplete
fn spawn_level_complete(
    mut commands: Commands,
    score: Res<Score>,
    current_level: Res<CurrentLevel>,
) {
    let score_val = score.value;
    let level_num = current_level.number + 1;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "LEVEL COMPLETE!", Color::srgb(0.3, 1.0, 0.4));
            spawn_subtitle(panel, &format!("Level {}  |  Score: {}", level_num, score_val));
            spawn_hint(panel, "[ ENTER - Next Level ]");
        });
    });
}

// ─── Оверлей паузы ──────────────────────────────────────────────────────────

fn update_pause_overlay(
    mut commands: Commands,
    paused: Res<Paused>,
    overlay_query: Query<Entity, With<PauseOverlay>>,
    state: Res<State<GameState>>,
) {
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
                    spawn_title(panel, "PAUSE", Color::srgb(0.9, 0.9, 0.3));
                    spawn_hint(panel, "[ ESC  Resume ]  [ ESC x2  Main Menu ]");
                });
            });
    } else {
        for entity in &overlay_query {
            commands.entity(entity).despawn_recursive();
        }
    }
}
