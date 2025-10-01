use bevy::prelude::*;

/// Resource tracking the current sanctuary state and configuration
#[derive(Resource, Debug, Clone)]
pub struct SanctuaryState {
    /// Sanctuary level/tier
    pub sanctuary_level: u32,
    /// Current depth reached (for progression tracking)
    pub current_depth: u32,
}

impl Default for SanctuaryState {
    fn default() -> Self {
        Self {
            sanctuary_level: 1,
            current_depth: 1,
        }
    }
}

impl SanctuaryState {
    /// Set the current depth
    pub fn set_depth(&mut self, depth: u32) {
        self.current_depth = depth;
    }

    /// Get the next dungeon depth to enter
    pub fn next_dungeon_depth(&self) -> u32 {
        self.current_depth + 1
    }
}
