use bevy::prelude::*;
use bevy::audio::{PlaybackSettings, Volume};

use crate::events::SoundEvent;
use crate::resources::assets::GameAssets;
use crate::resources::game_state::GameState;
use crate::resources::settings::AppSettings;

/// Маркер музыки главного меню
#[derive(Component)]
pub struct MenuMusicController;

/// Маркер музыки геймплея — не удаляется при смене уровня
#[derive(Component)]
pub struct MusicController;

/// Ресурс: музыка включена/выключена (F2)
#[derive(Resource)]
pub struct MusicEnabled(pub bool);

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SoundEvent>();

        // Вставляем ресурс немедленно через app.insert_resource() (не deferred),
        // иначе OnEnter(MainMenu) паникует — он срабатывает до flush команд Startup.
        // AssetServer доступен, т.к. DefaultPlugins уже добавлен перед AssetPlugin.
        // Загружаем хэндлы изображений/звуков
        let game_assets_partial = {
            let asset_server = app.world().resource::<AssetServer>();
            (
                asset_server.load("sounds/ball_hit.ogg"),
                asset_server.load("sounds/ball_hit.ogg"),
                asset_server.load("sounds/brick_hit.ogg"),
                asset_server.load("sounds/brick_break.ogg"),
                asset_server.load("sounds/bonus_pickup.ogg"),
                asset_server.load("sounds/life_lost.ogg"),
                asset_server.load("sounds/game_over.ogg"),
                asset_server.load("sounds/bullet_fire.ogg"),
                asset_server.load("sounds/ufo_hit.ogg"),
                asset_server.load("sounds/bomb_hit.ogg"),
                asset_server.load("music/menu.ogg"),
                asset_server.load("music/gameplay.ogg"),
                asset_server.load("sprites/padle.png"),
                asset_server.load::<Image>("sprites/ufo.png"),
                asset_server.load("sprites/ball.png"),
                asset_server.load("sprites/ball_fire.png"),
                asset_server.load("sprites/brick_normal.png"),
                asset_server.load("sprites/brick_strong.png"),
                asset_server.load("sprites/brick_strong_hit.png"),
                asset_server.load("sprites/bullet.png"),
                asset_server.load("sprites/bomb.png"),
                asset_server.load("sprites/bonus_paddle_grow.png"),
                asset_server.load("sprites/bonus_sticky.png"),
                asset_server.load("sprites/bonus_gun.png"),
                asset_server.load("sprites/bonus_ball_grow.png"),
                asset_server.load("sprites/bonus_fireball.png"),
                asset_server.load("sprites/bonus_multiball.png"),
                asset_server.load("sprites/Menu_back.png"),
                asset_server.load("sprites/Field_game.png"),
                asset_server.load("sprites/Field_game_sat.png"),
                asset_server.load("sprites/Field_Editor.png"),
                asset_server.load("fonts/PressStart2P-Regular.ttf"),
            )
        };

        // Создаём TextureAtlasLayout для НЛО (4 кадра 60×40 в ряд)
        let ufo_atlas_layout = {
            let layout = TextureAtlasLayout::from_grid(UVec2::new(60, 40), 4, 1, None, None);
            app.world_mut()
                .resource_mut::<Assets<TextureAtlasLayout>>()
                .add(layout)
        };

        let (
            sound_ball_wall, sound_ball_paddle, sound_ball_brick, sound_brick_break,
            sound_bonus_pickup, sound_life_lost, sound_game_over, sound_bullet_fire,
            sound_ufo_hit, sound_bomb_hit, music_menu, music_gameplay,
            sprite_paddle, sprite_ufo, sprite_ball, sprite_ball_fire,
            sprite_brick_normal, sprite_brick_strong, sprite_brick_strong_hit,
            sprite_bullet, sprite_bomb,
            sprite_bonus_paddle_grow, sprite_bonus_sticky, sprite_bonus_gun,
            sprite_bonus_ball_grow, sprite_bonus_fireball, sprite_bonus_multiball,
            bg_menu, bg_game, bg_game_sat, bg_editor,
            font_ui,
        ) = game_assets_partial;

        let game_assets = {
            GameAssets {
                sound_ball_wall, sound_ball_paddle, sound_ball_brick, sound_brick_break,
                sound_bonus_pickup, sound_life_lost, sound_game_over, sound_bullet_fire,
                sound_ufo_hit, sound_bomb_hit, music_menu, music_gameplay,
                sprite_paddle,
                sprite_ball, sprite_ball_fire,
                sprite_brick_normal, sprite_brick_strong, sprite_brick_strong_hit,
                sprite_ufo, ufo_atlas_layout,
                sprite_bullet, sprite_bomb,
                sprite_bonus_paddle_grow, sprite_bonus_sticky, sprite_bonus_gun,
                sprite_bonus_ball_grow, sprite_bonus_fireball, sprite_bonus_multiball,
                bg_menu, bg_game, bg_game_sat, bg_editor,
                font_ui,
            }
        };
        app.insert_resource(game_assets);
        app.insert_resource(MusicEnabled(true));

        // Музыка меню — играет в MainMenu, Options, HighScores, EnterName
        app.add_systems(OnEnter(GameState::MainMenu),   start_menu_music);
        app.add_systems(OnEnter(GameState::Options),    start_menu_music);
        app.add_systems(OnEnter(GameState::HighScores), start_menu_music);
        app.add_systems(OnEnter(GameState::EnterName),  start_menu_music);
        // Останавливаем музыку меню только при входе в игру/редактор
        app.add_systems(OnEnter(GameState::Playing),     stop_menu_music);
        app.add_systems(OnEnter(GameState::LevelEditor), stop_menu_music);

        // Музыка геймплея
        app.add_systems(OnEnter(GameState::Playing), start_gameplay_music);
        app.add_systems(OnEnter(GameState::MainMenu), stop_gameplay_music);

        // Воспроизведение звуков + управление музыкой
        app.add_systems(Update, (play_sounds_system, music_control_system).chain());
    }
}

// ─── Музыка ──────────────────────────────────────────────────────────────────

fn start_menu_music(
    mut commands: Commands,
    assets: Res<GameAssets>,
    query: Query<(), With<MenuMusicController>>,
) {
    if !query.is_empty() {
        return;
    }
    commands.spawn((
        MenuMusicController,
        AudioPlayer::new(assets.music_menu.clone()),
        PlaybackSettings::LOOP,
    ));
}

fn stop_menu_music(
    mut commands: Commands,
    query: Query<Entity, With<MenuMusicController>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn start_gameplay_music(
    mut commands: Commands,
    assets: Res<GameAssets>,
    query: Query<(), With<MusicController>>,
) {
    if !query.is_empty() {
        return;
    }
    commands.spawn((
        MusicController,
        AudioPlayer::new(assets.music_gameplay.clone()),
        PlaybackSettings::LOOP,
    ));
}

fn stop_gameplay_music(
    mut commands: Commands,
    query: Query<Entity, With<MusicController>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ─── Управление музыкой ──────────────────────────────────────────────────────

/// F2 — переключить музыку вкл/выкл; синхронизирует громкость и pause/resume каждый кадр
fn music_control_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut enabled: ResMut<MusicEnabled>,
    settings: Res<AppSettings>,
    mut sinks: Query<&mut AudioSink, Or<(With<MusicController>, With<MenuMusicController>)>>,
) {
    if keys.just_pressed(KeyCode::F2) {
        enabled.0 = !enabled.0;
    }

    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(settings.music_volume));
        if enabled.0 && sink.is_paused() {
            sink.play();
        } else if !enabled.0 && !sink.is_paused() {
            sink.pause();
        }
    }
}

// ─── Звуковые события ────────────────────────────────────────────────────────

fn play_sounds_system(
    mut commands: Commands,
    mut events: MessageReader<SoundEvent>,
    assets: Res<GameAssets>,
    settings: Res<AppSettings>,
) {
    let sfx_settings = PlaybackSettings {
        volume: Volume::Linear(settings.sfx_volume),
        ..PlaybackSettings::DESPAWN
    };
    for event in events.read() {
        let handle = match event {
            SoundEvent::BallHitWall    => assets.sound_ball_wall.clone(),
            SoundEvent::BallHitPaddle  => assets.sound_ball_paddle.clone(),
            SoundEvent::BallHitBrick   => assets.sound_ball_brick.clone(),
            SoundEvent::BrickBreak     => assets.sound_brick_break.clone(),
            SoundEvent::BonusPickup    => assets.sound_bonus_pickup.clone(),
            SoundEvent::LifeLost       => assets.sound_life_lost.clone(),
            SoundEvent::GameOver       => assets.sound_game_over.clone(),
            SoundEvent::BulletFire     => assets.sound_bullet_fire.clone(),
            SoundEvent::UfoHit         => assets.sound_ufo_hit.clone(),
            SoundEvent::BombHit        => assets.sound_bomb_hit.clone(),
        };
        commands.spawn((AudioPlayer::new(handle), sfx_settings));
    }
}
