use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for the player entity
#[derive(Component)]
pub struct Player;

/// First-person camera controller
#[derive(Component)]
pub struct PlayerCamera {
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

/// Player's flashlight
#[derive(Component)]
pub struct Flashlight {
    pub enabled: bool,
    pub intensity: f32,
    pub range: f32,
}

impl Default for Flashlight {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 1000.0,
            range: 20.0,
        }
    }
}

/// Marks entities/components that should be persisted
#[derive(Component, Serialize, Deserialize)]
pub struct Saveable;

/// Simple resource for testing save/load
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            maximum: 100.0,
        }
    }
}
