use rand::Rng;

use crate::constants::*;
use super::grid::{DungeonGrid, Direction};

/// Random walk algorithm to connect cells in the dungeon grid
pub fn random_walk_connect(
    grid: &mut DungeonGrid,
    rng: &mut impl Rng,
    start_x: usize,
    start_y: usize,
    fill_ratio: f32
) {
    let (width, height) = grid.dimensions();
    let target_cells = (width * height) as f32 * fill_ratio;
    let mut connected_cells = 0;
    let mut stack = Vec::new();

    // Start the walk
    grid.set_visited(start_x, start_y);
    connected_cells += 1;
    stack.push((start_x, start_y));

    while connected_cells < target_cells as usize && !stack.is_empty() {
        // Choose a random position from our stack (not always the last - more branching)
        let stack_idx = if stack.len() == 1 {
            0
        } else {
            rng.random_range(0..stack.len())
        };
        let (current_x, current_y) = stack[stack_idx];

        let unvisited_directions = grid.get_unvisited_directions(current_x, current_y);

        if !unvisited_directions.is_empty() {
            // Pick a random unvisited direction
            let direction = unvisited_directions[rng.random_range(0..unvisited_directions.len())];

            // Get the neighbor coordinates in that direction
            if let Some((next_x, next_y)) = grid.get_neighbor(current_x, current_y, direction) {
                // Connect the cells
                grid.connect_cells(current_x, current_y, direction);
                grid.set_visited(next_x, next_y);
                connected_cells += 1;

                // Add the new cell to our stack
                stack.push((next_x, next_y));
            }
        } else {
            // No more neighbors, remove this cell from the stack
            stack.remove(stack_idx);
        }
    }
}

/// Try to create a rectangular room by checking all cells in the defined room size area
fn try_create_rectangular_room(grid: &DungeonGrid, start_x: usize, start_y: usize) -> Vec<(usize, usize)> {
    let mut room_cells = Vec::new();

    // Check all cells in the room size area (ROOM_SIZE_X x ROOM_SIZE_Y)
    for dx in 0..ROOM_SIZE_X {
        for dy in 0..ROOM_SIZE_Y {
            let x = start_x + dx;
            let y = start_y + dy;

            // Check bounds
            if x >= grid.width() || y >= grid.height() {
                continue; // Skip cells outside grid
            }

            // Only include cells that are visited and not already in a room
            if grid.is_visited(x, y) && !grid.is_room(x, y) {
                room_cells.push((x, y));
            }
        }
    }

    room_cells
}

/// Connect all cells within a room to their adjacent room cells
fn connect_room_cells(grid: &mut DungeonGrid, room_cells: &[(usize, usize)]) {
    for &(x, y) in room_cells {
        // Check all 4 directions and connect to adjacent room cells

        // North
        if let Some((nx, ny)) = grid.get_neighbor(x, y, Direction::North) {
            if room_cells.contains(&(nx, ny)) {
                grid.connect_cells(x, y, Direction::North);
            }
        }

        // South
        if let Some((nx, ny)) = grid.get_neighbor(x, y, Direction::South) {
            if room_cells.contains(&(nx, ny)) {
                grid.connect_cells(x, y, Direction::South);
            }
        }

        // East
        if let Some((nx, ny)) = grid.get_neighbor(x, y, Direction::East) {
            if room_cells.contains(&(nx, ny)) {
                grid.connect_cells(x, y, Direction::East);
            }
        }

        // West
        if let Some((nx, ny)) = grid.get_neighbor(x, y, Direction::West) {
            if room_cells.contains(&(nx, ny)) {
                grid.connect_cells(x, y, Direction::West);
            }
        }
    }
}

/// Place rooms in some of the connected cells
pub fn place_rooms(grid: &mut DungeonGrid, rng: &mut impl Rng) {
    let room_attempts = ROOM_COUNT;
    let (width, height) = grid.dimensions();

    for current_room_id in 1..room_attempts + 1 {
        // Make 3 attempts to find a viable room starting position
        for _ in 0..3 {
            // Find a random visited cell that could become a room
            let room_x = rng.random_range(1..width - 1);
            let room_y = rng.random_range(1..height - 1);

            // Try to create a rectangular room starting from this position
            let room_cells = try_create_rectangular_room(grid, room_x, room_y);

            // Only create a room if we found enough eligible cells
            if room_cells.len() >= 4 {  // Minimum room size (at least 4 cells)
                // Convert all eligible cells to room cells
                for &(rx, ry) in &room_cells {
                    grid.set_room_with_id(rx, ry, current_room_id as u32);
                }

                // Connect all cells within the room to each other
                connect_room_cells(grid, &room_cells);

                break; // Successfully placed room, no need for more attempts
            }
        }
    }
}
