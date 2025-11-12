//! Transform gizmo system for interactive object manipulation
//!
//! This module provides a 3-axis transform gizmo (like Blender, Unity, etc.) that allows
//! users to visually manipulate selected objects in the 3D viewport.
//!
//! # Features
//!
//! - **Three transform modes**: Translate, Rotate, Scale (cycle with F key)
//! - **Two orientations**: Global (world-aligned) or Local (object-aligned) axes (toggle with O key)
//! - **Multi-select support**: Transform multiple objects simultaneously
//! - **Grid snapping**: Snap translations and rotations when grid snap is enabled
//! - **Visual feedback**: Handles highlight on hover
//! - **Speed modifiers**: Hold Shift for 4x speed, Ctrl for 0.25x speed during drag
//!
//! # Architecture
//!
//! The gizmo is spawned/despawned automatically based on object selection using
//! Bevy observers. Each gizmo consists of:
//! - A root entity (`GizmoRoot`) positioned at the selection center
//! - Three handle entities (`GizmoHandle`) for each axis (X, Y, Z)
//! - Visual mesh children (arrow shafts, cones, and axis lines)

pub mod systems;
pub mod types;

// Re-export commonly used types
pub use types::{GizmoState, GizmoHandle};

// Re-export all systems for plugin registration
pub use systems::{
    spawn_gizmo, despawn_gizmo, update_gizmo_position, toggle_transform_mode,
};
