use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::plugins::editor_plugin::EditorPlugin;
use crate::plugins::gameplay_plugin::GameplayPlugin;
use crate::plugins::level_plugin::LevelPlugin;
use crate::plugins::physics_plugin::PhysicsPlugin;
use crate::plugins::ui_plugin::UiPlugin;
use crate::resources::game_state::GameState;

pub fn build_app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Arkanoid — Rust + Bevy 0.15".to_string(),
                resolution: WindowResolution::new(800.0, 600.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }),
    );

    app.init_state::<GameState>();

    // Тёмно-синий фон — контраст для белого мяча и серых стен
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.15)));

    app.add_plugins((GameplayPlugin, PhysicsPlugin, UiPlugin, LevelPlugin, EditorPlugin));

    app
}
