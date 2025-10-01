//! Dungeon generation module
//!
//! This module provides functionality for procedural dungeon generation using
//! a grid-based approach with random walk algorithms and room placement.

pub mod grid;
pub mod generator;
pub mod spawner;

use bevy::prelude::*;
use rand::rng;

use crate::constants::*;
use super::scenes::manager::Scene;
use grid::DungeonGrid;
use generator::{random_walk_connect, place_rooms};
use spawner::{spawn_grid_as_entities, spawn_grid_as_entities_in_scene};

/// Generate and spawn a complete dungeon using the grid-based algorithm
pub fn generate_dungeon_rooms(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    _depth: u32,
) {
    let mut grid = DungeonGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut rng = rng();

    // Start random walk from center
    let start_x = GRID_WIDTH / 2;
    let start_y = GRID_HEIGHT / 2;

    // Random walk to connect cells
    random_walk_connect(&mut grid, &mut rng, start_x, start_y, 1.0);

    // Place rooms in some connected cells
    place_rooms(&mut grid, &mut rng);

    // Convert grid to physical entities
    spawn_grid_as_entities(commands, meshes, materials, &grid);
}

/// Scene-aware version of dungeon generation
pub fn generate_dungeon_rooms_in_scene<T: Scene>(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    _depth: u32,
) {
    let mut grid = DungeonGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut rng = rng();

    // Start random walk from center
    let start_x = GRID_WIDTH / 2;
    let start_y = GRID_HEIGHT / 2;

    // Random walk to connect cells
    random_walk_connect(&mut grid, &mut rng, start_x, start_y, 1.0);

    // Place rooms in some connected cells
    place_rooms(&mut grid, &mut rng);

    // Convert grid to physical entities with scene tracking
    spawn_grid_as_entities_in_scene::<T>(commands, meshes, materials, &grid);
}
