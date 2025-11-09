use bevy::prelude::*;
use bevy::pbr::MaterialPlugin;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

mod camera;
mod components;
mod grid;
mod placement;
mod primitives;
mod resources;
mod selection;
mod ui;

use camera::*;
use grid::*;
use placement::*;
use primitives::AssetCatalog;
use resources::EditorMouseMotion;
use selection::*;
use ui::*;

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

            // Custom material for grid
            .add_plugins(MaterialPlugin::<GridMaterial>::default())

            // Resources
            .init_resource::<EditorMouseMotion>()
            .init_resource::<AssetCatalog>()
            .init_resource::<PlacementState>()
            .init_resource::<SelectedEntity>()
            .init_resource::<GridConfig>()

            // Observers for picking events
            .add_observer(handle_selection)

            // Startup systems
            .add_systems(Startup, (
                setup_editor_camera,
                setup_test_scene,
                setup_grid,
                lock_cursor_on_start,
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
            ))
            // Update systems - UI (must run in EGUI pass)
            .add_systems(EguiPrimaryContextPass, (
                asset_browser_ui,
                inspector_ui,
                status_bar_ui,
            ));
    }
}
