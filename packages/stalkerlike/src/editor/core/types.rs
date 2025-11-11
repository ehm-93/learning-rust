use bevy::prelude::*;

/// Marker component for entities that belong to the editor scene
#[derive(Component)]
pub struct EditorEntity;

/// Marker component for player spawn point entities
/// Game mode can query for this to find where to spawn the player
#[derive(Component)]
pub struct PlayerSpawn;
