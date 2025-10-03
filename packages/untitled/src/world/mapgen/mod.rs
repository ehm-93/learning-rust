//! Map generation system for Phase 1.5
//!
//! This module implements the macro-level terrain generation system
//! that creates connected cave networks for each level.

pub mod freeform;
pub mod macro_map;
pub mod systems;
pub mod validation;
pub mod visualization;

// Re-export key types
pub use macro_map::{MacroMap, DEFAULT_MACRO_WIDTH, DEFAULT_MACRO_HEIGHT};
pub use freeform::generate_freeform_paths;
pub use validation::{flood_fill_validate, analyze_macro_map, MacroMapStats};
pub use systems::{generate_new_level, regenerate_current_level};
pub use visualization::{save_macro_map_png, save_detailed_macro_map_png, auto_save_macro_maps, manual_save_macro_map};

use bevy::prelude::*;

/// Plugin for map generation systems
pub struct MapGenPlugin;

impl Plugin for MapGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::initialize_level_resource)
           .add_systems(Update, (
               systems::regenerate_current_level,
               visualization::auto_save_macro_maps,
               visualization::manual_save_macro_map,
           ));
    }
}

/// Resource representing the current level and its macro map
#[derive(Resource)]
pub struct Level {
    /// Current level depth (1 = first level)
    pub depth: u32,
    /// Seed for this level's generation
    pub seed: u64,
    /// Macro map for this level (64x64 density grid)
    pub macro_map: MacroMap,
}

impl Level {
    /// Create a new level with the given depth and seed
    pub fn new(depth: u32, seed: u64) -> Self {
        let macro_map = MacroMap::generate(seed, depth);
        Self {
            depth,
            seed,
            macro_map,
        }
    }
}
