use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Game state machine
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

/// Save/load file path
#[derive(Resource)]
pub struct SavePath(pub PathBuf);

impl Default for SavePath {
    fn default() -> Self {
        Self(PathBuf::from("save.db"))
    }
}

/// Input state for camera control
#[derive(Resource, Default)]
pub struct MouseMotion {
    pub delta: Vec2,
}

/// Saved game data
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub position: Vec3,
    pub health: f32,
}
