//! Objects plugin for object lifecycle management

use bevy::prelude::*;

use super::box_select::{BoxSelectState, start_box_select, update_box_select, complete_box_select, cancel_box_select, render_box_select};
use super::duplication::handle_duplicate;
use super::gizmo::{GizmoState, spawn_gizmo, despawn_gizmo, update_gizmo_position, toggle_transform_mode};
use super::grouping::{GroupCounter, handle_group, handle_ungroup};
use super::outline::{spawn_outlines, despawn_outlines, sync_outline_transforms};
use super::placement::{PlacementState, update_preview_position, place_object};
use super::primitives::AssetCatalog;
use super::selection::{SelectionSet, SelectedEntity, handle_selection, handle_deselection, highlight_selected, remove_outline_from_deselected, delete_selected};

/// Plugin for object lifecycle management
///
/// Manages the complete lifecycle of objects in the editor:
/// - Primitives catalog (what can be placed)
/// - Placement system (creating new objects)
/// - Selection system (picking and multi-select)
/// - Gizmo system (transform manipulation)
/// - Grouping/ungrouping (hierarchical organization)
/// - Duplication/deletion operations
/// - Visual feedback (outlines, highlights)
pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app
            // ===== Resources =====
            .init_resource::<AssetCatalog>()
            .init_resource::<PlacementState>()
            .init_resource::<SelectionSet>()
            .init_resource::<SelectedEntity>() // Legacy - kept for backward compatibility
            .init_resource::<GizmoState>()
            .init_resource::<BoxSelectState>()
            .init_resource::<GroupCounter>()

            // ===== Observers =====
            // Selection events trigger gizmo spawn/despawn
            .add_observer(handle_selection)
            .add_observer(spawn_gizmo)
            .add_observer(despawn_gizmo)

            // ===== Update Systems =====

            // Placement systems - creating new objects
            .add_systems(Update, (
                update_preview_position,
                place_object,
            ))

            // Selection systems - picking and highlighting
            .add_systems(Update, (
                handle_deselection,
                delete_selected,
                highlight_selected,
                remove_outline_from_deselected,
                spawn_outlines,
                despawn_outlines,
                sync_outline_transforms,
            ))

            // Box selection - multi-select via drag
            .add_systems(Update, (
                start_box_select,
                update_box_select,
                complete_box_select,
                cancel_box_select,
                render_box_select,
            ))

            // Gizmo systems - transform manipulation
            .add_systems(Update, (
                update_gizmo_position,
                toggle_transform_mode,
            ))

            // Grouping systems - hierarchical organization
            .add_systems(Update, (
                handle_group,
                handle_ungroup,
            ))

            // Duplication system
            .add_systems(Update, handle_duplicate);
    }
}
