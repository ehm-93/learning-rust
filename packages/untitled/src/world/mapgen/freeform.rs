//! Freeform cave generation algorithm
//!
//! Implements random walks to create connected cave networks with organic shapes.
//! Based on the algorithm described in the map generation specification.

use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Generate connected cave paths using random walks
/// Modifies the grid in-place to carve out cave areas
///
/// Algorithm:
/// 1. Start from center with multiple random walks
/// 2. Use angular distribution to prevent overlap
/// 3. Apply cellular automata smoothing
/// 4. Add multi-scale noise variation
pub fn generate_freeform_paths(grid: &mut Vec<Vec<bool>>, seed: u64, level_depth: u32) {
    let width = grid[0].len();
    let height = grid.len();

    info!("Generating freeform paths for {}x{} grid (seed: {}, depth: {})", width, height, seed, level_depth);

    let mut rng = StdRng::seed_from_u64(seed);

    // Start from center
    let center_x = width / 2;
    let center_y = height / 2;

    // Carve out initial spawn area
    carve_area(grid, center_x, center_y, 3);

    // Generate multiple random walks from center
    let num_walks = 8 + (level_depth as usize * 2).min(16); // More paths on deeper levels
    let base_length = 20 + level_depth as usize * 5; // Longer paths on deeper levels

    for i in 0..num_walks {
        // Distribute walks in different directions to prevent overlap
        let angle = (i as f32 / num_walks as f32) * 2.0 * std::f32::consts::PI;
        let direction_bias = (angle.cos(), angle.sin());

        generate_random_walk(
            grid,
            center_x,
            center_y,
            base_length,
            direction_bias,
            &mut rng
        );
    }

    // Apply cellular automata smoothing to remove single-tile walls
    smooth_with_cellular_automata(grid, &mut rng);

    // Add micro-variation noise
    add_noise_variation(grid, seed, level_depth, &mut rng);

    info!("Completed freeform path generation");
}

/// Carve out a circular area around the given point
fn carve_area(grid: &mut Vec<Vec<bool>>, center_x: usize, center_y: usize, radius: usize) {
    let width = grid[0].len();
    let height = grid.len();

    for dy in -(radius as i32)..=(radius as i32) {
        for dx in -(radius as i32)..=(radius as i32) {
            let x = center_x as i32 + dx;
            let y = center_y as i32 + dy;

            if x >= 1 && y >= 1 && x < (width - 1) as i32 && y < (height - 1) as i32 {
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq <= (radius * radius) as f32 {
                    grid[y as usize][x as usize] = false; // Cave
                }
            }
        }
    }
}

/// Generate a single random walk path
fn generate_random_walk(
    grid: &mut Vec<Vec<bool>>,
    start_x: usize,
    start_y: usize,
    length: usize,
    direction_bias: (f32, f32),
    rng: &mut StdRng,
) {
    let width = grid[0].len();
    let height = grid.len();

    let mut x = start_x as f32;
    let mut y = start_y as f32;

    for _ in 0..length {
        // Carve current position
        let ix = x as usize;
        let iy = y as usize;

        if ix >= 1 && iy >= 1 && ix < width - 1 && iy < height - 1 {
            grid[iy][ix] = false;

            // Carve a small area around the point for thicker corridors
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = (ix as i32 + dx) as usize;
                    let ny = (iy as i32 + dy) as usize;
                    if nx < width && ny < height {
                        if rng.random_bool(0.7) { // 70% chance to carve adjacent cells
                            grid[ny][nx] = false;
                        }
                    }
                }
            }
        } else {
            break; // Hit boundary
        }

        // Choose next direction (biased toward the assigned direction)
        let random_angle = rng.random_range(0.0..2.0 * std::f32::consts::PI);
        let bias_strength = 0.3; // How much to bias toward assigned direction

        let final_dx = direction_bias.0 * bias_strength + random_angle.cos() * (1.0 - bias_strength);
        let final_dy = direction_bias.1 * bias_strength + random_angle.sin() * (1.0 - bias_strength);

        // Move with some randomness in step size
        let step_size = rng.random_range(0.5..1.5);
        x += final_dx * step_size;
        y += final_dy * step_size;
    }
}

/// Apply cellular automata to smooth out single-tile walls and caves
fn smooth_with_cellular_automata(grid: &mut Vec<Vec<bool>>, _rng: &mut StdRng) {
    let width = grid[0].len();
    let height = grid.len();

    // Apply multiple iterations of cellular automata
    for _iteration in 0..3 {
        let mut new_grid = grid.clone();

        for y in 1..height-1 {
            for x in 1..width-1 {
                let wall_neighbors = count_wall_neighbors(grid, x, y);

                // Cellular automata rules
                if wall_neighbors >= 5 {
                    new_grid[y][x] = true; // Become/stay wall
                } else if wall_neighbors <= 3 {
                    new_grid[y][x] = false; // Become/stay cave
                }
                // 4 neighbors: keep current state
            }
        }

        *grid = new_grid;
    }
}

/// Count wall neighbors in a 3x3 grid around the given position
fn count_wall_neighbors(grid: &Vec<Vec<bool>>, x: usize, y: usize) -> usize {
    let mut count = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx < 0 || ny < 0 || nx >= grid[0].len() as i32 || ny >= grid.len() as i32 {
                count += 1; // Out of bounds counts as wall
            } else if grid[ny as usize][nx as usize] {
                count += 1;
            }
        }
    }
    count
}

/// Add noise variation to create more organic shapes
fn add_noise_variation(grid: &mut Vec<Vec<bool>>, _seed: u64, level_depth: u32, rng: &mut StdRng) {
    let width = grid[0].len();
    let height = grid.len();

    // Add some randomness to make caves more organic
    let noise_intensity = 0.1 + (level_depth as f32 * 0.02); // Increase chaos with depth

    for y in 1..height-1 {
        for x in 1..width-1 {
            if rng.random_bool(noise_intensity as f64) {
                // Small chance to flip the cell
                grid[y][x] = !grid[y][x];
            }
        }
    }
}
