//! Map validation utilities
//!
//! Functions to validate that generated macro maps have proper connectivity
//! and meet gameplay requirements.

use bevy::prelude::*;
use std::collections::VecDeque;

/// Validate that all cave areas in the macro map are connected
/// Uses flood fill to check connectivity from the center
/// Returns true if the map is fully connected, false otherwise
pub fn flood_fill_validate(grid: &Vec<Vec<bool>>) -> bool {
    let width = grid[0].len();
    let height = grid.len();

    // Find starting point (center area)
    let start_x = width / 2;
    let start_y = height / 2;

    // Find the first open cell near center
    let mut start_pos = None;
    for radius in 0..width.min(height)/2 {
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                let x = start_x as i32 + dx;
                let y = start_y as i32 + dy;

                if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                    if !grid[y as usize][x as usize] {
                        start_pos = Some((x as usize, y as usize));
                        break;
                    }
                }
            }
            if start_pos.is_some() { break; }
        }
        if start_pos.is_some() { break; }
    }

    let Some((start_x, start_y)) = start_pos else {
        warn!("No open cells found in macro map!");
        return false;
    };

    // Count total open cells
    let total_open_cells = grid.iter()
        .flat_map(|row| row.iter())
        .filter(|&&cell| !cell)
        .count();

    if total_open_cells == 0 {
        warn!("No open cells in macro map!");
        return false;
    }

    // Flood fill to count reachable cells
    let reachable_cells = flood_fill_count(grid, start_x, start_y);

    let connectivity_ratio = reachable_cells as f32 / total_open_cells as f32;

    info!("Connectivity validation: {}/{} cells reachable ({:.1}%)",
          reachable_cells, total_open_cells, connectivity_ratio * 100.0);

    // Consider connected if at least 90% of open cells are reachable
    // This allows for some isolated small areas while ensuring main connectivity
    connectivity_ratio >= 0.9
}

/// Perform flood fill and count reachable open cells
fn flood_fill_count(grid: &Vec<Vec<bool>>, start_x: usize, start_y: usize) -> usize {
    let width = grid[0].len();
    let height = grid.len();

    let mut visited = vec![vec![false; width]; height];
    let mut queue = VecDeque::new();
    let mut count = 0;

    queue.push_back((start_x, start_y));
    visited[start_y][start_x] = true;

    while let Some((x, y)) = queue.pop_front() {
        count += 1;

        // Check 4-directional neighbors
        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];

        for (nx, ny) in neighbors {
            if nx < width && ny < height && !visited[ny][nx] && !grid[ny][nx] {
                visited[ny][nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    count
}

/// Calculate basic statistics about the macro map
pub fn analyze_macro_map(grid: &Vec<Vec<bool>>) -> MacroMapStats {
    let width = grid[0].len();
    let height = grid.len();
    let total_cells = width * height;

    let open_cells = grid.iter()
        .flat_map(|row| row.iter())
        .filter(|&&cell| !cell)
        .count();

    let wall_cells = total_cells - open_cells;
    let open_ratio = open_cells as f32 / total_cells as f32;

    // Count edge cells that are open (should be 0 for valid maps)
    let mut open_edge_cells = 0;
    for x in 0..width {
        if !grid[0][x] { open_edge_cells += 1; }
        if !grid[height-1][x] { open_edge_cells += 1; }
    }
    for y in 1..height-1 {
        if !grid[y][0] { open_edge_cells += 1; }
        if !grid[y][width-1] { open_edge_cells += 1; }
    }

    MacroMapStats {
        total_cells,
        open_cells,
        wall_cells,
        open_ratio,
        open_edge_cells,
        is_valid_boundary: open_edge_cells == 0,
    }
}

/// Statistics about a macro map
#[derive(Debug)]
pub struct MacroMapStats {
    pub total_cells: usize,
    pub open_cells: usize,
    pub wall_cells: usize,
    pub open_ratio: f32,
    pub open_edge_cells: usize,
    pub is_valid_boundary: bool,
}
