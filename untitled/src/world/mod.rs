//! World setup and management
//!
//! This module is responsible for world initialization and configuration.

pub mod dungeon;
pub mod scenes;
pub mod interaction;

// Re-export interaction components for easy access
pub use interaction::{
    Interactable, InteractionEvent, InteractableHighlight, InteractionCallback,
};

// WorldPlugin is defined below and automatically exported as pub

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    constants::*,
    resources::*,
    world::scenes::manager::{SceneManager, SceneId},
    world::scenes::cathedral::CathedralScene,
    player::{Player, FireTimer},
};

/// Disables gravity for the 2D physics world
pub fn disable_gravity(mut query: Query<&mut RapierConfiguration>) {
    for mut config in &mut query {
        config.gravity = Vec2::ZERO;
    }
}



/// Reset game to Cathedral scene - used for restart functionality
pub fn reset_to_cathedral(
    mut scene_manager: ResMut<scenes::SceneManager>,
    mut score: ResMut<Score>,
    mut game_state: ResMut<GameState>,
    mut fire_timer: ResMut<FireTimer>,
) {
    // Reset game state
    *game_state = GameState::Playing;
    score.current = 0;

    // Reset timers
    fire_timer.timer.reset();

    // Clear all scenes and return to Cathedral
    scene_manager.clear_scenes();
    scene_manager.push_scene(scenes::CathedralScene::new());
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

/// Plugin that organizes all world-related systems, events, and resources
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Events
            .add_event::<InteractionEvent>()

            // Resources
            .insert_resource(scenes::SceneManager::new())

            // Systems
            .add_systems(Update, (
                // Scene management
                scenes::manager::update_scenes,

                // Consolidated interaction systems
                interaction::update_interactable_cooldowns,
                interaction::update_hovered_interactable, // Manages HoveredInteractable marker component
                interaction::handle_basic_interactions.run_if(resource_equals(GameState::Playing)),
                interaction::update_interactable_highlights,
                interaction::manage_halo_effects, // Now handles all halo management
            ))

            // Setup initial scene
            .add_systems(Startup, setup_initial_scene);
    }
}

/// Setup initial scene (Cathedral)
fn setup_initial_scene(mut scene_manager: ResMut<scenes::SceneManager>) {
    scene_manager.push_scene(scenes::CathedralScene::new());
}


