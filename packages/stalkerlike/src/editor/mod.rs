use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy default plugins
            .add_plugins(DefaultPlugins)

            // Third-party plugins
            .add_plugins(EguiPlugin::default());
    }
}
