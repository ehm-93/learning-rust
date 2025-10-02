use bevy::prelude::*;



/// Score tracking resource
#[derive(Resource, Default)]
pub struct Score {
    pub current: u32,
}

/// Game state resource to track if game is over
#[derive(Resource, Default, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

/// Game mode resource to track if we're in Cathedral or Dungeon
#[derive(Resource, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Cathedral,
    Dungeon,
}

/// Dungeon generation parameters
#[derive(Resource)]
pub struct DungeonParams {
    // Fields removed as they were unused - keeping struct for future expansion
}

impl Default for DungeonParams {
    fn default() -> Self {
        Self {
        }
    }
}
