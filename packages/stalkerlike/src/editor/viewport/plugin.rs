//! Viewport plugin for 3D scene viewing and navigation

use bevy::prelude::*;

use super::camera::{setup_editor_camera, toggle_mouse_lock, camera_look, camera_movement, lock_cursor_on_start};
use super::grid::{GridConfig, setup_grid, toggle_snap};

/// Plugin for viewport functionality (camera, grid, raycasting)
pub struct ViewportPlugin;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GridConfig>()

            // Startup systems
            .add_systems(Startup, (
                setup_editor_camera,
                setup_grid,
                lock_cursor_on_start,
            ))

            // Update systems
            .add_systems(Update, (
                toggle_mouse_lock,
                camera_look,
                camera_movement,
                toggle_snap,
            ));
    }
}
