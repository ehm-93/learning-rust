use bevy::prelude::*;

/// Main world state enum for game scenes
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldState {
    // System states
    MainMenu,

    // Game world states
    Cathedral,
    Sanctuary,
    Dungeon,
}

impl Default for WorldState {
    fn default() -> Self {
        WorldState::Cathedral
    }
}

/// Portal identifier - re-exported from cathedral components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortalId {
    Left,
    Center,
    Right,
}

/// Reason for exiting a dungeon
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExitReason {
    PlayerChoice,   // escaped successfully
    Death,          // died and respawned
}

/// Player progression data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerProgress {
    pub max_depth_reached: u32,
    pub max_depth_extracted_from: u32,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self {
            max_depth_reached: 0,
            max_depth_extracted_from: 0,
        }
    }
}

impl PlayerProgress {
    pub fn with_depth_reached(depth: u32) -> Self {
        Self {
            max_depth_reached: depth,
            max_depth_extracted_from: 0,
        }
    }

    pub fn update_depth_reached(&mut self, depth: u32) {
        self.max_depth_reached = self.max_depth_reached.max(depth);
    }

    pub fn update_depth_extracted(&mut self, depth: u32) {
        self.max_depth_extracted_from = self.max_depth_extracted_from.max(depth);
        self.update_depth_reached(depth);
    }
}

/// Resource for dungeon configuration data
#[derive(Resource, Debug, Clone)]
pub struct DungeonConfig {
    pub depth: u32,
    pub modifiers: Vec<String>, // Using String for now, can refactor to ModifierId later
    pub portal_id: PortalId,
}

impl Default for DungeonConfig {
    fn default() -> Self {
        Self {
            depth: 1,
            modifiers: Vec::new(),
            portal_id: PortalId::Left,
        }
    }
}

/// Resource for cathedral player progress
#[derive(Resource, Debug, Clone)]
pub struct CathedralConfig {
    pub player_progress: PlayerProgress,
}

impl Default for CathedralConfig {
    fn default() -> Self {
        Self {
            player_progress: PlayerProgress::default(),
        }
    }
}

/// Helper functions for checking state variants
impl WorldState {
    pub fn is_cathedral(&self) -> bool {
        matches!(self, WorldState::Cathedral)
    }

    pub fn is_dungeon(&self) -> bool {
        matches!(self, WorldState::Dungeon)
    }
}
