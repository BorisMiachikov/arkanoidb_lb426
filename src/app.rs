use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::plugins::asset_plugin::AssetPlugin;
use crate::plugins::editor_plugin::EditorPlugin;
use crate::plugins::gameplay_plugin::GameplayPlugin;
use crate::plugins::level_plugin::LevelPlugin;
use crate::plugins::physics_plugin::PhysicsPlugin;
use crate::plugins::ui_plugin::UiPlugin;
use crate::resources::game_state::GameState;
use crate::resources::settings::AppSettings;

pub fn build_app() -> App {
    // Загружаем настройки до создания окна, чтобы применить сохранённый размер
    let settings = AppSettings::load();
    let win_w = settings.window_width;
    let win_h = settings.window_height;

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Arkanoid — Rust + Bevy 0.15".to_string(),
                resolution: WindowResolution::new(win_w, win_h),
                resizable: true,
                ..default()
            }),
            ..default()
        }),
    );

    app.init_state::<GameState>();

    // Тёмно-синий фон — контраст для белого мяча и серых стен
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.15)));

    // Вставляем настройки до плагинов, чтобы они не перезаписывали Default
    app.insert_resource(settings);

    app.add_plugins((AssetPlugin, GameplayPlugin, PhysicsPlugin, UiPlugin, LevelPlugin, EditorPlugin));

    app
}
