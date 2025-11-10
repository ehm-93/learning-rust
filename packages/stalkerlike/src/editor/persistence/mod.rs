//! Scene persistence system for saving and loading editor scenes
//!
//! This module provides an extensible component-based serialization system
//! for editor scenes. The design follows these principles:
//!
//! - **By-component serialization**: Each component type can be serialized independently
//! - **Extensible**: New component types can be added without modifying core serialization
//! - **Human-readable**: Uses YAML for version-control friendly diffs
//! - **Type-safe**: Strongly typed Rust structs with serde validation
//!
//! # Current Support
//!
//! - Transform (position, rotation, scale)
//! - Mesh3d (primitive type)
//! - MeshMaterial3d (base color only for now)
//!
//! # Future Extensibility
//!
//! Additional component types can be added by:
//! 1. Creating a serializable struct in `scene.rs`
//! 2. Implementing `ComponentSerializer` trait
//! 3. Registering in `SerializerRegistry`

pub mod scene;
pub mod systems;

// Re-export for use in editor plugin
pub use systems::CurrentFile;
