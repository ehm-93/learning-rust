//! Objects domain - Object lifecycle management
//!
//! This domain handles the complete lifecycle of objects in the editor:
//! - Primitive definitions and catalog (what objects can be created)
//! - Object placement system (creating new objects)
//! - Object selection system (picking and highlighting)
//! - Transform manipulation (gizmos)
//! - Future: duplication, deletion

pub mod gizmo;
pub mod outline;
pub mod placement;
pub mod plugin;
pub mod primitives;
pub mod selection;

pub use plugin::ObjectsPlugin;
