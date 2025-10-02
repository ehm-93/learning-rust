//! Map generation systems
//!
//! This module contains the tilemap generation and management systems,
//! starting with static tilemaps and building towards procedural generation.

pub mod tiles;
pub mod systems;
pub mod resources;

// Re-export key types
pub use tiles::*;
pub use systems::*;
pub use resources::*;

use bevy::prelude::*;

/// Plugin for map generation - Static tilemap with basic collision
pub struct MapgenPlugin;

impl Plugin for MapgenPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add startup systems for loading assets
            .add_systems(Startup, systems::load_tilemap_texture);
            // Note: tilemap spawning is now handled by individual scenes
    }
}
