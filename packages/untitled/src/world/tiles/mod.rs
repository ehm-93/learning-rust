//! Tile systems
//!
//! This module contains the tilemap generation and management systems,
//! starting with static tilemaps and building towards procedural generation.

pub mod tiles;
pub mod systems;
pub mod resources;

// Re-export key types
pub use tiles::*;
pub use resources::*;

use bevy::prelude::*;

/// Plugin for tile systems - Static tilemap with basic collision
pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            // Add startup systems for loading assets
            .add_systems(Startup, systems::load_tilemap_texture);
            // Note: chunk management is now handled by ChunkPlugin
    }
}
