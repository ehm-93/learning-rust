use bevy::prelude::*;
use std::path::PathBuf;

/// Game state machine
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    NewGame,
    Loading,
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

/// Event to request saving the game
#[derive(Event)]
pub struct SaveGameEvent {
    pub slot: u32,
}

/// Event to request loading the game
#[derive(Event)]
pub struct LoadGameEvent {
    pub slot: u32,
}

/// Resource to track loading progress
#[derive(Resource, Default)]
pub struct LoadProgress {
    pub registered: Vec<String>,
    pub completed: Vec<String>,
}

impl LoadProgress {
    pub fn register(&mut self, system_name: impl Into<String>) {
        self.registered.push(system_name.into());
    }

    pub fn complete(&mut self, system_name: impl Into<String>) {
        self.completed.push(system_name.into());
    }

    pub fn is_complete(&self) -> bool {
        self.registered.len() == self.completed.len()
    }

    pub fn progress(&self) -> f32 {
        if self.registered.is_empty() {
            return 1.0;
        }
        self.completed.len() as f32 / self.registered.len() as f32
    }

    pub fn current_system(&self) -> Option<&str> {
        if self.is_complete() {
            None
        } else {
            self.registered.get(self.completed.len()).map(|s| s.as_str())
        }
    }
}
