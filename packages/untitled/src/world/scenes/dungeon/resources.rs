use bevy::prelude::*;

/// Resource tracking the current dungeon state and configuration
#[derive(Resource, Debug, Clone)]
pub struct DungeonState {
    /// Current dungeon depth/level
    pub depth: u32,

    /// Total rooms in this dungeon level
    pub total_rooms: u32,

    /// Rooms that have been cleared
    pub cleared_rooms: u32,

    /// Whether the dungeon is completed
    pub is_completed: bool,

    /// Difficulty multiplier based on depth
    pub difficulty_multiplier: f32,

    /// Available loot tier for this depth
    pub loot_tier: u32,

    pub macro_map: Vec<Vec<bool>>,
}

impl Default for DungeonState {
    fn default() -> Self {
        Self {
            depth: 1,
            total_rooms: 3, // Simple dungeon with 3 rooms
            cleared_rooms: 0,
            is_completed: false,
            difficulty_multiplier: 1.0,
            loot_tier: 1,
            macro_map: vec![],
        }
    }
}

impl DungeonState {
    pub fn new_for_depth(depth: u32) -> Self {
        Self {
            depth,
            total_rooms: 3 + (depth / 2), // More rooms as depth increases
            cleared_rooms: 0,
            is_completed: false,
            difficulty_multiplier: 1.0 + (depth as f32 * 0.2), // 20% harder per depth
            loot_tier: 1 + (depth / 3), // Better loot every 3 depths
            macro_map: vec![],
        }
    }

    pub fn room_cleared(&mut self) {
        self.cleared_rooms += 1;
        if self.cleared_rooms >= self.total_rooms {
            self.is_completed = true;
        }
    }
}
