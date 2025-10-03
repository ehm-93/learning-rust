//! World setup and management
//!
//! This module is responsible for world initialization and state-based scene management.

pub mod scenes;
pub mod interaction;
pub mod states;
pub mod tiles;
pub mod chunks;

// Re-export key types for easy access
pub use interaction::{
    Interactable, InteractionEvent, InteractableHighlight, InteractionCallback,
};
pub use states::WorldState;
pub use tiles::{WallTile};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    resources::*,
    player::Player,
};

/// Disables gravity for the 2D physics world
pub fn disable_gravity(mut query: Query<&mut RapierConfiguration>) {
    for mut config in &mut query {
        config.gravity = Vec2::ZERO;
    }
}

/// Cleans up game entities (enemies, projectiles) for restart
pub fn cleanup_game_entities(
    commands: &mut Commands,
    entities_query: &Query<Entity, (Or<(With<Enemy>, With<Projectile>)>, Without<Player>, Without<MainCamera>)>,
) {
    for entity in entities_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Cleans up all dungeon entities (walls, floors) for complete regeneration
pub fn cleanup_dungeon_entities(
    commands: &mut Commands,
    dungeon_query: &Query<Entity, With<DungeonWall>>,
    floor_query: &Query<Entity, (With<Mesh2d>, Without<Player>, Without<MainCamera>, Without<DungeonWall>, Without<Enemy>)>,
) {
    // Remove all dungeon walls
    for entity in dungeon_query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove all floor tiles (this is a bit of a hack - we're removing all mesh entities that aren't player/camera/walls/enemies)
    // In a more complex game, we'd want to tag floor tiles explicitly
    for entity in floor_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Plugin that organizes all world-related systems using state-based scene management
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize world state
            .init_state::<WorldState>()

            // Events
            .add_event::<InteractionEvent>()

            // Tile and chunk plugins
            .add_plugins((
                tiles::TilePlugin,
                chunks::ChunkPlugin,
            ))

            // Add scene plugins (each handles their own OnEnter/OnExit transitions)
            .add_plugins((
                scenes::cathedral::CathedralPlugin,
                scenes::sanctuary::SanctuaryPlugin,
                scenes::dungeon::DungeonPlugin,
            ))

            // Global interaction systems (run regardless of scene)
            .add_systems(Update, (
                // First update cooldowns and highlights (based on player/cursor position)
                interaction::update_interactable_cooldowns,
                interaction::update_interactable_highlights,
                // Then update which interactable is hovered (depends on highlights)
                interaction::update_hovered_interactable,
                // Then handle interactions (depends on hovered state)
                interaction::handle_basic_interactions.run_if(resource_equals(GameState::Playing)),
                // Finally manage visual effects
                interaction::manage_halo_effects,
                interaction::cleanup_orphaned_halos,
            ).chain());
    }
}


