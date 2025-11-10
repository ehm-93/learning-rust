//! Editor plugin for the stalkerlike game
//!
//! This module provides a complete 3D editor for level design and prototyping.
//! The editor is organized into domain-based modules:
//!
//! - [`core`] - Shared fundamentals (types, materials)
//! - [`viewport`] - 3D scene viewing (camera, grid, raycasting)
//! - [`objects`] - Object lifecycle (primitives, placement, selection)
//! - [`ui`] - User interface panels (asset browser, inspector, status bar)
//! - [`input`] - Input handling (mouse, keyboard shortcuts)
//! - [`persistence`] - Scene save/load system
//!
//! # Usage
//!
//! ```no_run
//! use bevy::prelude::*;
//! use stalkerlike::editor::EditorPlugin;
//!
//! App::new()
//!     .add_plugins(EditorPlugin)
//!     .run();
//! ```

use bevy::prelude::*;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::winit::WinitWindows;

// Domain modules
mod core;
mod input;
mod objects;
mod persistence;
mod ui;
mod viewport;

// Import domain plugins
use core::CorePlugin;
use input::InputPlugin;
use objects::ObjectsPlugin;
use persistence::PersistencePlugin;
use ui::UiPlugin;
use viewport::ViewportPlugin;

/// Main editor plugin that aggregates all domain plugins
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy default plugins
            .add_plugins(DefaultPlugins)

            // Picking plugin (mesh raycasting backend)
            .add_plugins(MeshPickingPlugin)

            // Domain plugins (order matters for some dependencies)
            .add_plugins(CorePlugin)         // Materials and shared types
            .add_plugins(InputPlugin)        // Input abstraction
            .add_plugins(ViewportPlugin)     // Camera and grid
            .add_plugins(ObjectsPlugin)      // Object lifecycle
            .add_plugins(PersistencePlugin)  // Save/load
            .add_plugins(UiPlugin)           // UI panels (depends on persistence events)

            // Startup systems
            .add_systems(Startup, maximize_window);
    }
}

/// Maximize the window on startup
fn maximize_window(
    windows: Query<Entity, With<Window>>,
    winit_windows: NonSend<WinitWindows>,
) {
    for entity in windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            winit_window.set_maximized(true);
        }
    }
}
