//! Core plugin providing shared editor fundamentals

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
