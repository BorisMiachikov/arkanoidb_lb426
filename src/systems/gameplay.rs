use bevy::app::AppExit;
use bevy::prelude::*;

use crate::components::ball::{Ball, BallStuck};
use crate::components::brick::Brick;
use crate::components::velocity::Velocity;
use crate::resources::game_state::GameState;
use crate::resources::level_data::LEVELS;
use crate::resources::editor::EditorData;
use crate::events::SoundEvent;
use crate::resources::score::{AudioSettings, CurrentLevel, DebugSkipPending, HighScore, Lives, MenuSelection, OptionsSelection, Paused, Score};
use crate::setup::level::HALF_H;

/// Главное меню: навигация и выбор
pub fn handle_main_menu_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selection: ResMut<MenuSelection>,
    mut editor: ResMut<EditorData>,
    mut app_exit: EventWriter<AppExit>,
) {
    // Навигация: вверх/вниз
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        if selection.0 > 0 {
            selection.0 -= 1;
        }
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        selection.0 = (selection.0 + 1).min(3);
    }

    // Подтвердить
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        match selection.0 {
            0 => {
                editor.active = false;
                next_state.set(GameState::Playing);
            }
            1 => next_state.set(GameState::LevelEditor),
            2 => next_state.set(GameState::Options),
            3 => { app_exit.send(AppExit::Success); }
            _ => {}
        }
    }
}

/// Пауза: первый ESC → пауза; второй ESC → выход в главное меню
pub fn handle_pause_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if paused.0 {
            paused.0 = false;
            next_state.set(GameState::MainMenu);
        } else {
            paused.0 = true;
        }
    }
}

/// Мячи упали за нижнюю границу.
/// Дополнительные мячи (MultiBall) просто удаляются без потери жизни.
/// Жизнь теряется только когда упал ПОСЛЕДНИЙ мяч на поле.
pub fn check_ball_lost_system(
    mut commands: Commands,
    mut ball_query: Query<(Entity, &mut Transform, &mut Velocity), With<Ball>>,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    // Собираем упавших и считаем всего мячей
    let total = ball_query.iter().count();
    let mut fallen: Vec<Entity> = Vec::new();

    for (ball_entity, transform, _) in &ball_query {
        if transform.translation.y < -(HALF_H + 20.0) {
            fallen.push(ball_entity);
        }
    }

    if fallen.is_empty() {
        return;
    }

    let surviving = total - fallen.len();

    if surviving > 0 {
        // На поле ещё есть мячи — просто удаляем упавшие
        for entity in fallen {
            commands.entity(entity).despawn();
        }
    } else {
        // Последний мяч потерян — теряем жизнь
        lives.count = lives.count.saturating_sub(1);

        if lives.count == 0 {
            sound_events.send(SoundEvent::GameOver);
            next_state.set(GameState::GameOver);
        } else {
            sound_events.send(SoundEvent::LifeLost);
            // Оставляем один мяч прилипшим к ракетке, остальные удаляем
            let mut first = true;
            for (ball_entity, mut transform, mut velocity) in &mut ball_query {
                if first {
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    transform.translation.x = 0.0;
                    commands.entity(ball_entity).insert(BallStuck);
                    first = false;
                } else {
                    commands.entity(ball_entity).despawn();
                }
            }
        }
    }
}

/// Все блоки уничтожены — победа на уровне.
pub fn check_win_condition_system(
    brick_query: Query<Entity, With<Brick>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() == GameState::Playing && brick_query.is_empty() {
        next_state.set(GameState::LevelComplete);
    }
}

/// Следит за счётом и обновляет рекорд при его превышении
pub fn track_highscore_system(score: Res<Score>, mut highscore: ResMut<HighScore>) {
    if score.is_changed() && score.value > highscore.value {
        highscore.value = score.value;
        highscore.save();
    }
}

/// В состоянии GameOver: Enter/Space → рестарт с первого уровня
pub fn handle_game_over_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut current_level: ResMut<CurrentLevel>,
    mut editor: ResMut<EditorData>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        score.value = 0;
        lives.count = 3;
        current_level.number = 0;
        editor.active = false;
        next_state.set(GameState::Playing);
    }
}

/// DEBUG: NumpadMultiply (*) → переход на следующий уровень через LevelComplete,
/// чтобы OnExit(Playing) запустил cleanup_level, а OnEnter(Playing) — spawn_level_entities.
pub fn debug_skip_level_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
    mut skip_pending: ResMut<DebugSkipPending>,
) {
    if keys.just_pressed(KeyCode::NumpadMultiply) {
        current_level.number += 1;
        if current_level.number as usize >= LEVELS.len() {
            current_level.number = 0;
        }
        skip_pending.0 = true;
        next_state.set(GameState::LevelComplete);
    }
}

/// DEBUG: автоматически продолжает из LevelComplete если выставлен флаг DebugSkipPending
pub fn debug_auto_advance_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut skip_pending: ResMut<DebugSkipPending>,
) {
    if skip_pending.0 {
        skip_pending.0 = false;
        next_state.set(GameState::Playing);
    }
}

/// Options: навигация W/S, изменение Left/Right, ESC/Enter(Back) → MainMenu
pub fn handle_options_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selection: ResMut<OptionsSelection>,
    mut settings: ResMut<AudioSettings>,
) {
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        if selection.0 > 0 { selection.0 -= 1; }
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        selection.0 = (selection.0 + 1).min(2);
    }

    let delta = if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        -0.1f32
    } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        0.1f32
    } else {
        0.0
    };

    if delta != 0.0 {
        match selection.0 {
            0 => settings.music_volume = (settings.music_volume + delta).clamp(0.0, 1.0),
            1 => settings.sfx_volume   = (settings.sfx_volume   + delta).clamp(0.0, 1.0),
            _ => {}
        }
    }

    let back = keys.just_pressed(KeyCode::Escape)
        || ((keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
            && selection.0 == 2);
    if back {
        next_state.set(GameState::MainMenu);
    }
}

/// В состоянии LevelComplete: Enter/Space → следующий уровень
pub fn handle_level_complete_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        current_level.number += 1;
        // После последнего уровня — начинаем сначала
        if current_level.number as usize >= LEVELS.len() {
            current_level.number = 0;
        }
        next_state.set(GameState::Playing);
    }
}
