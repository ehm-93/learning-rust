//! Editor plugin for the stalkerlike game
//!
//! This module provides a complete 3D editor for level design and prototyping.
//! The editor is organized into domain-based modules:
//!
//! - [`viewport`] - 3D scene viewing (camera, grid, raycasting)
//! - [`objects`] - Object lifecycle (primitives, placement, selection)
//! - [`ui`] - User interface panels (asset browser, inspector, status bar)
//! - [`input`] - Input handling (mouse, keyboard shortcuts)
//! - [`core`] - Shared fundamentals (types, materials)
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
use bevy::pbr::MaterialPlugin;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::winit::WinitWindows;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

// Domain modules
mod core;
mod input;
mod objects;
mod ui;
mod viewport;

// Import types and systems from domains
use core::materials::GridMaterial;
use input::mouse::EditorMouseMotion;
use objects::{
    gizmo::{GizmoState, GizmoMaterial, spawn_gizmo, despawn_gizmo, update_gizmo_position, toggle_transform_mode},
    outline::{OutlineMaterial, spawn_outlines, despawn_outlines, sync_outline_transforms},
    placement::{PlacementState, update_preview_position, place_object},
    primitives::AssetCatalog,
    selection::{SelectedEntity, handle_selection, handle_deselection, highlight_selected, remove_outline_from_deselected},
};
use viewport::{
    camera::{setup_editor_camera, toggle_mouse_lock, camera_look, camera_movement, lock_cursor_on_start},
    grid::{GridConfig, setup_grid, toggle_snap},
};
use ui::{
    asset_browser::asset_browser_ui,
    inspector::inspector_ui,
    status_bar::status_bar_ui,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy default plugins
            .add_plugins(DefaultPlugins)

            // Third-party plugins
            .add_plugins(EguiPlugin::default())

            // Picking plugin (mesh raycasting backend)
            .add_plugins(MeshPickingPlugin)

            // Custom materials
            .add_plugins(MaterialPlugin::<GridMaterial>::default())
            .add_plugins(MaterialPlugin::<GizmoMaterial>::default())

            // Resources
            .init_resource::<EditorMouseMotion>()
            .init_resource::<AssetCatalog>()
            .init_resource::<PlacementState>()
            .init_resource::<SelectedEntity>()
            .init_resource::<GridConfig>()
            .init_resource::<GizmoState>()
            .init_resource::<OutlineMaterial>()

            // Observers for picking events and component changes
            .add_observer(handle_selection)
            .add_observer(spawn_gizmo)    // OnAdd<Selected>
            .add_observer(despawn_gizmo)  // OnRemove<Selected>

            // Startup systems
            .add_systems(Startup, (
                setup_editor_camera,
                setup_grid,
                lock_cursor_on_start,
                maximize_window,
            ))

            // Update systems - camera
            .add_systems(Update, (
                toggle_mouse_lock,
                camera_look,
                camera_movement,
            ))
            // Update systems - grid
            .add_systems(Update, toggle_snap)
            // Update systems - placement
            .add_systems(Update, (
                update_preview_position,
                place_object,
            ))
            // Update systems - selection
            .add_systems(Update, (
                handle_deselection,
                highlight_selected,
                remove_outline_from_deselected,
                spawn_outlines,
                despawn_outlines,
                sync_outline_transforms,
            ))
            // Update systems - gizmo
            .add_systems(Update, (
                update_gizmo_position,
                toggle_transform_mode,
            ))
            // Update systems - UI (must run in EGUI pass)
            .add_systems(EguiPrimaryContextPass, (
                asset_browser_ui,
                inspector_ui,
                status_bar_ui,
            ));
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
