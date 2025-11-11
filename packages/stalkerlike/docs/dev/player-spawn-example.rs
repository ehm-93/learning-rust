// Example: Using the Player Spawn Marker in Game Mode
// This file demonstrates how to query for and use the player spawn marker

use bevy::prelude::*;

// Import the PlayerSpawn component from the editor
// (In a real implementation, this might be in a shared module)
use crate::editor::core::types::PlayerSpawn;

/// System to spawn the player at the marked spawn point
pub fn spawn_player_at_marker(
    mut commands: Commands,
    spawn_query: Query<&Transform, With<PlayerSpawn>>,
    // ... other resources for player entity creation
) {
    // Find the player spawn marker in the scene
    let spawn_transform = spawn_query
        .get_single()
        .expect("Scene should have exactly one PlayerSpawn marker");
    
    let spawn_position = spawn_transform.translation;
    let spawn_rotation = spawn_transform.rotation;
    
    info!("Spawning player at position: {:?}", spawn_position);
    
    // Spawn the player entity at the marked position
    commands.spawn((
        // Player component
        Player,
        // Use the spawn marker's transform
        Transform {
            translation: spawn_position,
            rotation: spawn_rotation,
            scale: Vec3::ONE,
        },
        // ... other player components (mesh, collider, controller, etc.)
    ));
}

/// More flexible version that handles missing or multiple spawn points
pub fn spawn_player_at_marker_flexible(
    mut commands: Commands,
    spawn_query: Query<&Transform, With<PlayerSpawn>>,
) {
    // Default spawn if no marker is found
    let default_spawn = Transform::from_xyz(0.0, 1.0, 0.0);
    
    // Try to find a spawn marker
    let spawn_transform = if let Ok(transform) = spawn_query.get_single() {
        // Found exactly one spawn marker - use it
        *transform
    } else if spawn_query.is_empty() {
        // No spawn markers - use default
        warn!("No PlayerSpawn marker found in scene, using default position");
        default_spawn
    } else {
        // Multiple spawn markers - use the first one and warn
        warn!("Multiple PlayerSpawn markers found, using the first one");
        *spawn_query.iter().next().unwrap()
    };
    
    info!("Spawning player at: {:?}", spawn_transform.translation);
    
    // Spawn player
    commands.spawn((
        Player,
        spawn_transform,
        // ... other components
    ));
}

/// Example: Named spawn points (future enhancement)
/// This demonstrates how you might extend the system with named spawns
#[derive(Component)]
pub struct NamedSpawnPoint {
    pub name: String,
    pub priority: i32,
}

pub fn spawn_player_at_named_point(
    mut commands: Commands,
    spawn_query: Query<(&Transform, &NamedSpawnPoint)>,
    desired_spawn_name: Option<&str>,
) {
    let spawn_transform = if let Some(name) = desired_spawn_name {
        // Find spawn point by name
        spawn_query
            .iter()
            .find(|(_, spawn)| spawn.name == name)
            .map(|(transform, _)| *transform)
            .unwrap_or_else(|| {
                warn!("Spawn point '{}' not found, using default", name);
                Transform::from_xyz(0.0, 1.0, 0.0)
            })
    } else {
        // No specific spawn requested - use highest priority
        spawn_query
            .iter()
            .max_by_key(|(_, spawn)| spawn.priority)
            .map(|(transform, _)| *transform)
            .unwrap_or_else(|| Transform::from_xyz(0.0, 1.0, 0.0))
    };
    
    commands.spawn((Player, spawn_transform));
}

// Placeholder Player component for the example
#[derive(Component)]
struct Player;
