//! Input plugin for input handling and action mapping

use bevy::prelude::*;

use super::mouse::EditorMouseMotion;

/// Plugin for input abstraction (mouse, keyboard)
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<EditorMouseMotion>();
    }
}
