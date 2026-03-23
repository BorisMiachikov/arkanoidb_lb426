use bevy::prelude::*;

use crate::components::bonus_effects::{BallGrowEffect, GunPaddleEffect, PaddleGrowEffect, StickyEffect};
use crate::resources::game_state::GameState;
use crate::resources::score::{CurrentLevel, Lives, Paused, Score};

// ─── Маркеры HUD ────────────────────────────────────────────────────────────

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct LivesText;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct ActiveBonusText;

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
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu);
        app.add_systems(OnExit(GameState::MainMenu), despawn_overlay);

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
                update_lives_ui,
                update_level_ui,
                update_bonus_ui,
                update_pause_overlay,
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
                    Text::new("LIVES: 3"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                    LivesText,
                ));
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

fn update_lives_ui(lives: Res<Lives>, mut query: Query<&mut Text, With<LivesText>>) {
    if lives.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            **text = format!("LIVES: {}", lives.count);
        }
    }
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

    **text = bonuses.join("  ");
}

// ─── Оверлеи состояний ──────────────────────────────────────────────────────

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

// Главное меню
fn spawn_main_menu(mut commands: Commands) {
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "ARKANOID", Color::srgb(0.3, 0.7, 1.0));
            spawn_subtitle(panel, "Rust  +  Bevy  Edition");
            spawn_hint(panel, "[ ENTER or SPACE to Start ]");
        });
    });
}

// GameOver
fn spawn_game_over(mut commands: Commands, score: Res<Score>) {
    let score_val = score.value;
    let root = spawn_overlay_root(&mut commands);
    commands.entity(root).with_children(|parent| {
        spawn_panel(parent, |panel| {
            spawn_title(panel, "GAME OVER", Color::srgb(1.0, 0.25, 0.25));
            spawn_subtitle(panel, &format!("Score: {}", score_val));
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
                    spawn_hint(panel, "[ ESC to Resume ]");
                });
            });
    } else {
        for entity in &overlay_query {
            commands.entity(entity).despawn_recursive();
        }
    }
}
