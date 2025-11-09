//! Objects domain - Object lifecycle management
//!
//! This domain handles the complete lifecycle of objects in the editor:
//! - Primitive definitions and catalog (what objects can be created)
//! - Object placement system (creating new objects)
//! - Object selection system (picking and highlighting)
//! - Future: Transform manipulation (gizmos), duplication, deletion

pub mod placement;
pub mod primitives;
pub mod selection;
