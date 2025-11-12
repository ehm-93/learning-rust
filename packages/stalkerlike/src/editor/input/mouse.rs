//! Mouse input tracking and abstraction
//!
//! This module provides a resource for tracking editor-specific mouse state,
//! decoupling mouse input from the rest of the editor systems.
//!
//! Currently tracks mouse motion delta, which is used by the camera system
//! for mouse look functionality.

use bevy::prelude::*;

/// Input state for editor camera control
#[derive(Resource, Default)]
pub struct EditorMouseMotion {
    pub delta: Vec2,
}
