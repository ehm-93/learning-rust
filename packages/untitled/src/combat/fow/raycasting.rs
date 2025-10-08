//! Raycasting utilities for line-of-sight calculations
//!
//! Provides Bresenham's line algorithm implementation for wall-based
//! line-of-sight blocking in the fog of war system.

use bevy::prelude::*;
use std::collections::HashMap;

/// Safety limit for raycasting to prevent infinite loops
const RAYCAST_MAX_STEPS: i32 = 1000;

/// Check if there's a wall at the given world tile coordinates
///
/// # Arguments
/// * `terrain` - Map of chunk positions to wall data (row-major: `[y][x]`)
/// * `world_x` - X coordinate in world tile space
/// * `world_y` - Y coordinate in world tile space
///
/// # Returns
/// `true` if there's a wall at the position, `false` for floors or missing chunks
pub fn is_wall_at(
    terrain: &HashMap<IVec2, Vec<Vec<bool>>>,
    world_x: i32,
    world_y: i32,
    chunk_size: usize,
) -> bool {
    let chunk_x = world_x.div_euclid(chunk_size as i32);
    let chunk_y = world_y.div_euclid(chunk_size as i32);
    let local_x = world_x.rem_euclid(chunk_size as i32) as usize;
    let local_y = world_y.rem_euclid(chunk_size as i32) as usize;

    terrain
        .get(&IVec2::new(chunk_x, chunk_y))
        .and_then(|chunk| chunk.get(local_y))
        .and_then(|row| row.get(local_x))
        .copied()
        .unwrap_or(false) // Treat missing chunks as passable (not walls)
}

/// Cast a ray using Bresenham's line algorithm to check line of sight
///
/// This function traces a line from the start point to the end point,
/// checking for walls along the way. It returns the distance traveled
/// before hitting a wall, or the full distance if the path is clear.
///
/// This allows for soft blending near walls by comparing the ray distance
/// to the actual distance to the target tile.
///
/// # Arguments
/// * `terrain` - Map of chunk positions to wall data
/// * `from_x`, `from_y` - Starting position in world tile coordinates
/// * `to_x`, `to_y` - Target position in world tile coordinates
/// * `chunk_size` - Size of terrain chunks in tiles
///
/// # Returns
/// Distance traveled before hitting a wall, or full distance if clear
pub fn cast_ray_distance(
    terrain: &HashMap<IVec2, Vec<Vec<bool>>>,
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
    chunk_size: usize,
) -> f32 {
    let dx = (to_x - from_x).abs();
    let dy = (to_y - from_y).abs();
    let sx = if from_x < to_x { 1 } else { -1 };
    let sy = if from_y < to_y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = from_x;
    let mut y = from_y;
    let mut step_count = 0;

    // Calculate total distance to target
    let total_distance = ((to_x - from_x) as f32).hypot((to_y - from_y) as f32);

    loop {
        // Don't check the starting position
        if x != from_x || y != from_y {
            if is_wall_at(terrain, x, y, chunk_size) {
                // Hit a wall - return distance traveled so far
                let current_distance = ((x - from_x) as f32).hypot((y - from_y) as f32);
                return current_distance;
            }
        }

        // Reached the destination without hitting a wall
        if x == to_x && y == to_y {
            return total_distance;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
        step_count += 1;

        // Safety check to prevent infinite loops
        if step_count > RAYCAST_MAX_STEPS {
            return total_distance;
        }
    }
}
