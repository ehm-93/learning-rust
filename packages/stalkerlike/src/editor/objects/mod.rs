//! Objects domain - Object lifecycle management
//!
//! This domain handles the complete lifecycle of objects in the editor:
//! - Primitive definitions and catalog (what objects can be created)
//! - Object placement system (creating new objects)
//! - Object selection system (picking and highlighting)
//! - Box selection system (multi-select via drag)
//! - Transform manipulation (gizmos)
//! - Grouping/ungrouping (hierarchical organization)
//! - Duplication/deletion operations

pub mod box_select;
pub mod duplication;
pub mod gizmo;
pub mod grouping;
pub mod outline;
pub mod placement;
pub mod plugin;
pub mod primitives;
pub mod selection;

// Re-export plugin
pub use plugin::ObjectsPlugin;

// Re-export commonly used types for other modules
pub use selection::{SelectionSet, Selected};
pub use gizmo::GizmoState;
pub use placement::{PlacementState, PlacementAsset};
pub use primitives::{AssetCatalog, PrimitiveDefinition, PrimitiveType};
pub use outline::Outlined;
