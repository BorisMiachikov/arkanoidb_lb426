use bevy::prelude::*;

use crate::resources::editor::EditorData;
use crate::resources::game_state::GameState;
use crate::systems::editor::{
    cleanup_editor, editor_keyboard_system, editor_mouse_system, editor_redraw_system,
    setup_editor,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorData>();

        app.add_systems(OnEnter(GameState::LevelEditor), setup_editor);
        app.add_systems(OnExit(GameState::LevelEditor), cleanup_editor);

        app.add_systems(
            Update,
            (
                editor_mouse_system,
                editor_keyboard_system,
                editor_redraw_system,
            )
                .chain()
                .run_if(in_state(GameState::LevelEditor)),
        );
    }
}
