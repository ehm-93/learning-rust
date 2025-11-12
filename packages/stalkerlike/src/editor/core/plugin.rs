//! Core plugin providing shared editor fundamentals

//! Core plugin providing shared editor fundamentals
//!
//! Registers custom materials and initializes core resources that are
//! used across multiple editor domains.

use bevy::prelude::*;
use bevy::pbr::MaterialPlugin;

use super::materials::{GridMaterial, GizmoMaterial, OutlineMaterial};

/// Plugin for core editor functionality (materials, types)
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register custom materials
            .add_plugins(MaterialPlugin::<GridMaterial>::default())
            .add_plugins(MaterialPlugin::<GizmoMaterial>::default())

            // Initialize material resources
            .init_resource::<OutlineMaterial>();
    }
}
