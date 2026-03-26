use bevy::prelude::*;
use bevy::audio::PlaybackSettings;

use crate::events::SoundEvent;
use crate::resources::assets::GameAssets;
use crate::resources::game_state::GameState;

/// Маркер музыки главного меню
#[derive(Component)]
pub struct MenuMusicController;

/// Маркер музыки геймплея — не удаляется при смене уровня
#[derive(Component)]
pub struct MusicController;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEvent>();

        // Вставляем ресурс немедленно через app.insert_resource() (не deferred),
        // иначе OnEnter(MainMenu) паникует — он срабатывает до flush команд Startup.
        // AssetServer доступен, т.к. DefaultPlugins уже добавлен перед AssetPlugin.
        let game_assets = {
            let asset_server = app.world().resource::<AssetServer>();
            GameAssets {
                sound_ball_wall:    asset_server.load("sounds/ball_hit.ogg"),
                sound_ball_paddle:  asset_server.load("sounds/ball_hit.ogg"),
                sound_ball_brick:   asset_server.load("sounds/brick_hit.ogg"),
                sound_brick_break:  asset_server.load("sounds/brick_break.ogg"),
                sound_bonus_pickup: asset_server.load("sounds/bonus_pickup.ogg"),
                sound_life_lost:    asset_server.load("sounds/life_lost.ogg"),
                sound_game_over:    asset_server.load("sounds/game_over.ogg"),
                sound_bullet_fire:  asset_server.load("sounds/bullet_fire.ogg"),
                sound_ufo_hit:      asset_server.load("sounds/ufo_hit.ogg"),
                sound_bomb_hit:     asset_server.load("sounds/bomb_hit.ogg"),
                music_menu:         asset_server.load("music/menu.ogg"),
                music_gameplay:     asset_server.load("music/gameplay.ogg"),
                sprite_paddle:            asset_server.load("sprites/paddle.png"),
                sprite_ball:              asset_server.load("sprites/ball.png"),
                sprite_ball_fire:         asset_server.load("sprites/ball_fire.png"),
                sprite_brick_normal:      asset_server.load("sprites/brick_normal.png"),
                sprite_brick_strong:      asset_server.load("sprites/brick_strong.png"),
                sprite_brick_strong_hit:  asset_server.load("sprites/brick_strong_hit.png"),
                sprite_ufo:               asset_server.load("sprites/ufo.png"),
                sprite_bullet:            asset_server.load("sprites/bullet.png"),
                sprite_bomb:              asset_server.load("sprites/bomb.png"),
                sprite_bonus_paddle_grow: asset_server.load("sprites/bonus_paddle_grow.png"),
                sprite_bonus_sticky:      asset_server.load("sprites/bonus_sticky.png"),
                sprite_bonus_gun:         asset_server.load("sprites/bonus_gun.png"),
                sprite_bonus_ball_grow:   asset_server.load("sprites/bonus_ball_grow.png"),
                sprite_bonus_fireball:    asset_server.load("sprites/bonus_fireball.png"),
                sprite_bonus_multiball:   asset_server.load("sprites/bonus_multiball.png"),
            }
        };
        app.insert_resource(game_assets);

        // Музыка меню
        app.add_systems(OnEnter(GameState::MainMenu), start_menu_music);
        app.add_systems(OnExit(GameState::MainMenu), stop_menu_music);

        // Музыка геймплея
        app.add_systems(OnEnter(GameState::Playing), start_gameplay_music);
        app.add_systems(OnEnter(GameState::MainMenu), stop_gameplay_music);

        // Воспроизведение звуков
        app.add_systems(Update, play_sounds_system);
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

// ─── Звуковые события ────────────────────────────────────────────────────────

fn play_sounds_system(
    mut commands: Commands,
    mut events: EventReader<SoundEvent>,
    assets: Res<GameAssets>,
) {
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
        commands.spawn((AudioPlayer::new(handle), PlaybackSettings::DESPAWN));
    }
}
