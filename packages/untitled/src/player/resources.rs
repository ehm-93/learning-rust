use bevy::prelude::*;

/// Timer resource for controlling player fire rate
#[derive(Resource)]
pub struct FireTimer {
    pub timer: Timer,
}

impl Default for FireTimer {
    fn default() -> Self {
        use crate::constants::*;
        Self {
            timer: Timer::from_seconds(FIRE_RATE, TimerMode::Once),
        }
    }
}

/// Configuration resource for player behavior
#[derive(Resource)]
pub struct PlayerConfig {
    /// Player movement speed multiplier
    pub movement_speed_multiplier: f32,
    /// Camera follow smoothing
    pub camera_smoothing: f32,
    /// Cursor bias strength for camera
    pub cursor_bias_strength: f32,
    /// Maximum cursor bias distance
    pub cursor_bias_max_distance: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            movement_speed_multiplier: 1.0,
            camera_smoothing: 0.05,
            cursor_bias_strength: 0.5,
            cursor_bias_max_distance: 300.0,
        }
    }
}

/// Resource for managing camera zoom level across scenes
#[derive(Resource)]
pub struct CameraZoom {
    /// Current zoom level (higher values = more zoomed out)
    pub level: f32,
    /// Minimum zoom level (closest zoom)
    pub min_zoom: f32,
    /// Maximum zoom level (furthest zoom)
    pub max_zoom: f32,
    /// Zoom sensitivity (how much each scroll step changes zoom)
    pub sensitivity: f32,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self {
            level: 1.5,      // Start at 1.5x zoom out
            min_zoom: 3.0,   // Can zoom in to 0.5x (closer)
            max_zoom: 3.0,   // Can zoom out to 3.0x (further)
            sensitivity: 0.1, // Each scroll step changes zoom by 0.1
        }
    }
}
