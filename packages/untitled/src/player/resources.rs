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
            camera_smoothing: 0.1,
            cursor_bias_strength: 0.3,
            cursor_bias_max_distance: 200.0,
        }
    }
}
