use bevy::prelude::*;

/// Все загруженные ассеты игры: звуки и спрайты.
/// Хэндлы готовы сразу при старте — Bevy загружает ассеты асинхронно.
/// Если файл отсутствует — звук не воспроизводится, спрайт не отображается.
#[derive(Resource)]
pub struct GameAssets {
    // ── Звуки ────────────────────────────────────────────────────────────────
    pub sound_ball_wall:    Handle<AudioSource>,
    pub sound_ball_paddle:  Handle<AudioSource>,
    pub sound_ball_brick:   Handle<AudioSource>,
    pub sound_brick_break:  Handle<AudioSource>,
    pub sound_bonus_pickup: Handle<AudioSource>,
    pub sound_life_lost:    Handle<AudioSource>,
    pub sound_game_over:    Handle<AudioSource>,
    pub sound_bullet_fire:  Handle<AudioSource>,
    pub sound_ufo_hit:      Handle<AudioSource>,
    pub sound_bomb_hit:     Handle<AudioSource>,
    // ── Музыка ───────────────────────────────────────────────────────────────
    pub music_menu:         Handle<AudioSource>,
    pub music_gameplay:     Handle<AudioSource>,
    // ── Спрайты (используются при переходе на спрайтовый рендер) ─────────────
    pub sprite_paddle:            Handle<Image>,
    pub sprite_ball:              Handle<Image>,
    pub sprite_ball_fire:         Handle<Image>,
    pub sprite_brick_normal:      Handle<Image>,
    pub sprite_brick_strong:      Handle<Image>,
    pub sprite_brick_strong_hit:  Handle<Image>,
    pub sprite_ufo:               Handle<Image>,
    pub ufo_atlas_layout:         Handle<TextureAtlasLayout>,
    pub sprite_bullet:            Handle<Image>,
    pub sprite_bomb:              Handle<Image>,
    pub sprite_bonus_paddle_grow: Handle<Image>,
    pub sprite_bonus_sticky:      Handle<Image>,
    pub sprite_bonus_gun:         Handle<Image>,
    pub sprite_bonus_ball_grow:   Handle<Image>,
    pub sprite_bonus_fireball:    Handle<Image>,
    pub sprite_bonus_multiball:   Handle<Image>,
    // ── Фоны ─────────────────────────────────────────────────────────────────
    pub bg_menu:     Handle<Image>,
    pub bg_game:     Handle<Image>,
    pub bg_game_sat: Handle<Image>,
    pub bg_editor:   Handle<Image>,
    // ── Шрифт UI ─────────────────────────────────────────────────────────────
    pub font_ui: Handle<Font>,
}
