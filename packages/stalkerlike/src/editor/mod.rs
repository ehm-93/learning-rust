use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod camera;
mod components;
mod resources;

use camera::*;
use resources::EditorMouseMotion;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy default plugins
            .add_plugins(DefaultPlugins)

            // Third-party plugins
            .add_plugins(EguiPlugin::default())

            // Resources
            .init_resource::<EditorMouseMotion>()

            // Startup systems
            .add_systems(Startup, (
                setup_editor_camera,
                setup_test_scene,
                lock_cursor_on_start,
            ))

            // Update systems
            .add_systems(Update, (
                toggle_mouse_lock,
                camera_look,
                camera_movement,
            ));
    }
}
