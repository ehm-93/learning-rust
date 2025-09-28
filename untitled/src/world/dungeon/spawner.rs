use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::DungeonWall,
    constants::*,
};
use super::grid::DungeonGrid;

/// Convert the logical grid into physical entities
pub fn spawn_grid_as_entities(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &DungeonGrid,
) {
    let (width, height) = grid.dimensions();

    // Calculate the physical world bounds
    let total_width = width as f32 * CELL_SIZE;
    let total_height = height as f32 * CELL_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;

    // For each cell in the grid, spawn walls for unvisited cells or floor tiles for visited cells
    for x in 0..width {
        for y in 0..height {
            let world_x = start_x + (x as f32 + 0.5) * CELL_SIZE;
            let world_y = start_y + (y as f32 + 0.5) * CELL_SIZE;

            if !grid.is_visited(x, y) {
                // Spawn solid wall for unvisited cells
                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE))),
                    MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
                    Transform::from_translation(Vec3::new(world_x, world_y, 0.0)),
                    DungeonWall,
                    RigidBody::Fixed,
                    Collider::cuboid(CELL_SIZE / 2.0, CELL_SIZE / 2.0),
                ));
            } else {
                // Spawn floor tile for visited cells
                let floor_color = if grid.is_room(x, y) {
                    Color::srgb(ROOM_FLOOR_COLOR[0], ROOM_FLOOR_COLOR[1], ROOM_FLOOR_COLOR[2])
                } else {
                    Color::srgb(CORRIDOR_FLOOR_COLOR[0], CORRIDOR_FLOOR_COLOR[1], CORRIDOR_FLOOR_COLOR[2])
                };

                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE))),
                    MeshMaterial2d(materials.add(floor_color)),
                    Transform::from_translation(Vec3::new(world_x, world_y, -1.0)), // Slightly behind walls
                ));
            }
        }
    }

    // Spawn walls between all adjacent unconnected cells (avoiding duplicates)
    spawn_all_walls(commands, meshes, materials, grid);
}

/// Spawn walls between all adjacent unconnected cells, avoiding duplicates
pub fn spawn_all_walls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &DungeonGrid,
) {
    let (width, height) = grid.dimensions();
    let total_width = width as f32 * CELL_SIZE;
    let total_height = height as f32 * CELL_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;
    let wall_thickness = WALL_THICKNESS;
    let half_cell = CELL_SIZE / 2.0;

    // For each cell, check right and down neighbors to avoid duplicate walls
    for x in 0..width {
        for y in 0..height {
            let world_x = start_x + (x as f32 + 0.5) * CELL_SIZE;
            let world_y = start_y + (y as f32 + 0.5) * CELL_SIZE;

            let current_visited = grid.is_visited(x, y);
            let current_room_id = grid.get_room_id(x, y);
            let connections = grid.get_connections(x, y);

            // Check east neighbor (right)
            if x + 1 < width {
                let neighbor_visited = grid.is_visited(x + 1, y);
                let neighbor_room_id = grid.get_room_id(x + 1, y);

                // Spawn wall if:
                // 1. No connection between cells, OR
                // 2. Different rooms (both are rooms), OR
                // 3. One cell is visited and the other isn't
                let should_spawn_wall = !connections.east ||
                    (current_visited != neighbor_visited) ||
                    (current_room_id > 0 && neighbor_room_id > 0 && current_room_id != neighbor_room_id);

                if should_spawn_wall {
                    spawn_wall(
                        commands,
                        meshes,
                        materials,
                        world_x + half_cell,
                        world_y,
                        wall_thickness,
                        CELL_SIZE,
                        true, // vertical wall
                    );
                }
            } else if current_visited {
                // Edge of grid = wall for visited cells
                spawn_wall(
                    commands,
                    meshes,
                    materials,
                    world_x + half_cell,
                    world_y,
                    wall_thickness,
                    CELL_SIZE,
                    true, // vertical wall
                );
            }

            // Check north neighbor (up)
            if y + 1 < height {
                let neighbor_visited = grid.is_visited(x, y + 1);
                let neighbor_room_id = grid.get_room_id(x, y + 1);

                // Same logic as east wall
                let should_spawn_wall = !connections.north ||
                    (current_visited != neighbor_visited) ||
                    (current_room_id > 0 && neighbor_room_id > 0 && current_room_id != neighbor_room_id);

                if should_spawn_wall {
                    spawn_wall(
                        commands,
                        meshes,
                        materials,
                        world_x,
                        world_y + half_cell,
                        CELL_SIZE,
                        wall_thickness,
                        false, // horizontal wall
                    );
                }
            } else if current_visited {
                // Edge of grid = wall for visited cells
                spawn_wall(
                    commands,
                    meshes,
                    materials,
                    world_x,
                    world_y + half_cell,
                    CELL_SIZE,
                    wall_thickness,
                    false, // horizontal wall
                );
            }

            // Handle left and bottom edges for visited cells
            if x == 0 && current_visited {
                // Left edge wall
                spawn_wall(
                    commands,
                    meshes,
                    materials,
                    world_x - half_cell,
                    world_y,
                    wall_thickness,
                    CELL_SIZE,
                    true, // vertical wall
                );
            }

            if y == 0 && current_visited {
                // Bottom edge wall
                spawn_wall(
                    commands,
                    meshes,
                    materials,
                    world_x,
                    world_y - half_cell,
                    CELL_SIZE,
                    wall_thickness,
                    false, // horizontal wall
                );
            }
        }
    }
}

/// Helper function to spawn a single wall entity
fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    _is_vertical: bool,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(width, height))),
        MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
        Transform::from_translation(Vec3::new(x, y, 0.0)),
        DungeonWall,
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, height / 2.0),
    ));
}
