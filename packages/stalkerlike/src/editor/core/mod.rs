//! Core domain - Shared editor fundamentals
//!
//! This domain contains types and utilities shared across all editor domains:
//! - Common component types (EditorEntity marker)
//! - Custom materials (GridMaterial with distance-based fade)
//! - Future: Common utilities, editor configuration

pub mod materials;
pub mod plugin;
pub mod types;

// Re-export plugin
pub use plugin::CorePlugin;

// Re-export commonly used types for other modules
pub use types::{EditorEntity, PlayerSpawn, GlbModel, RigidBodyType, MissingAsset, EditorLight, EditorVisualization, LightType};
pub use materials::{GridMaterial, GizmoMaterial, OutlineMaterial};
