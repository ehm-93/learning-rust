//! Systems for map generation
//!
//! Bevy systems that handle level creation and macro map management.

use bevy::prelude::*;
use super::{Level, DEFAULT_MACRO_WIDTH, DEFAULT_MACRO_HEIGHT};

/// Initialize the Level resource on startup
/// Generates the first level with a default seed
pub fn initialize_level_resource(mut commands: Commands) {
    let seed = 12345; // Default seed, will be configurable later
    let level = Level::new(1, seed);

    info!("Initialized level {} with seed {}", level.depth, level.seed);
    info!("Open cells: {}/{} ({:.1}%)",
          level.macro_map.count_open_cells(),
          level.macro_map.width * level.macro_map.height,
          level.macro_map.count_open_cells() as f32 / (level.macro_map.width * level.macro_map.height) as f32 * 100.0);

    commands.insert_resource(level);
}

/// Generate a new level with the given parameters
/// This can be called when the player descends stairs
pub fn generate_new_level(
    commands: &mut Commands,
    depth: u32,
    seed: Option<u64>,
    width: Option<usize>,
    height: Option<usize>
) {
    let actual_seed = seed.unwrap_or_else(|| {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let actual_width = width.unwrap_or(DEFAULT_MACRO_WIDTH);
    let actual_height = height.unwrap_or(DEFAULT_MACRO_HEIGHT);

    info!("Generating new level: depth={}, seed={}, size={}x{}", depth, actual_seed, actual_width, actual_height);

    let mut level = Level::new(depth, actual_seed);
    if actual_width != DEFAULT_MACRO_WIDTH || actual_height != DEFAULT_MACRO_HEIGHT {
        level.macro_map = super::MacroMap::generate_sized(actual_width, actual_height, actual_seed, depth);
    }

    commands.insert_resource(level);
}

/// System to regenerate the current level (useful for testing)
pub fn regenerate_current_level(
    mut commands: Commands,
    level: Option<Res<Level>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        if let Some(current_level) = level {
            info!("Regenerating level {} (F5 pressed)", current_level.depth);
            generate_new_level(&mut commands, current_level.depth, None, None, None);
        }
    }
}
