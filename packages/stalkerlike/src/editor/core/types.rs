use bevy::prelude::*;
use std::path::PathBuf;

/// Marker component for entities that belong to the editor scene
#[derive(Component)]
pub struct EditorEntity;

/// Marker component for player spawn point entities
/// Game mode can query for this to find where to spawn the player
#[derive(Component)]
pub struct PlayerSpawn;

/// Component for entities that have a GLB/GLTF model
/// Stores the path to the model file
#[derive(Component, Clone, Debug)]
pub struct GlbModel {
    /// Path to the GLB/GLTF file (relative to asset directory or absolute)
    pub path: PathBuf,
}

/// Editor representation of rigid body type
/// Serialized with scenes and converted to actual Rapier RigidBody in game mode
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RigidBodyType {
    /// Static body - does not move, good for level geometry
    #[default]
    Fixed,
    /// Dynamic body - moves and is affected by physics
    Dynamic,
}

impl RigidBodyType {
    /// Get all variants for UI display
    pub fn variants() -> &'static [RigidBodyType] {
        &[RigidBodyType::Fixed, RigidBodyType::Dynamic]
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            RigidBodyType::Fixed => "Fixed",
            RigidBodyType::Dynamic => "Dynamic",
        }
    }
}
