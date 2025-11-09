use bevy::prelude::*;

/// Editor camera controller with fly-around controls
/// Attach to the same entity as Camera3d - this just adds editor-specific behavior
#[derive(Component)]
pub struct EditorCamera {
    /// Mouse look sensitivity
    pub sensitivity: f32,
    /// Vertical rotation (pitch) in radians
    pub pitch: f32,
    /// Horizontal rotation (yaw) in radians
    pub yaw: f32,
    /// Current velocity for smooth movement
    pub velocity: Vec3,
    /// Base movement speed (units per second)
    pub base_speed: f32,
    /// Whether mouse is locked for camera control (true) or free for UI interaction (false)
    pub mouse_locked: bool,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            base_speed: 5.0,
            mouse_locked: true,
        }
    }
}

/// Marker component for entities that belong to the editor scene
#[derive(Component)]
pub struct EditorEntity;
