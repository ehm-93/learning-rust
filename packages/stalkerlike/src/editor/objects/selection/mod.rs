//! Object selection system
//!
//! This module handles selecting and deselecting objects in the editor viewport.
//!
//! # Features
//!
//! - **Click to select**: Click any EditorEntity to select it
//! - **Multi-select**: Hold Ctrl and click to add/remove entities from selection
//! - **Visual feedback**: Selected entities are highlighted with an outline
//! - **Keyboard shortcuts**: ESC to deselect all, Delete to delete selected
//! - **Locked entities**: Cannot select entities marked as Locked
//!
//! # Selection Set
//!
//! The `SelectionSet` resource tracks all currently selected entities. This supports:
//! - Multi-entity operations (transform, delete, group, etc.)
//! - Querying selection state
//! - Adding/removing entities from selection programmatically
//!
//! # Integration with Other Systems
//!
//! - **Gizmo**: Spawns when entities are selected (OnAdd<Selected> observer)
//! - **Outline**: Adds visual highlight to selected entities
//! - **Placement**: Prevents selection during object placement mode
//! - **Hierarchy**: UI displays selection state, respects lock status

pub mod systems;
pub mod types;

// Re-export commonly used types
pub use types::{SelectionSet, Selected, SelectedEntity};

// Re-export all systems for plugin registration
pub use systems::{
    handle_selection, handle_deselection, highlight_selected,
    remove_outline_from_deselected, delete_selected,
};
