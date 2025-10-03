//! Wall boundary detection and polyline generation
//!
//! This module provides algorithms to:
//! 1. Flood fill connected wall regions in a chunk
//! 2. Trace boundaries around wall regions to create polylines
//! 3. Generate efficient polygon colliders from these boundaries

use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};
use crate::world::{
    chunks::CHUNK_SIZE,
    tiles::TileType,
};

/// A point representing a tile coordinate within a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePoint {
    pub x: u32,
    pub y: u32,
}

impl TilePoint {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Get neighboring points (4-directional)
    pub fn neighbors(&self) -> Vec<TilePoint> {
        let mut neighbors = Vec::new();

        // Up
        if self.y < CHUNK_SIZE - 1 {
            neighbors.push(TilePoint::new(self.x, self.y + 1));
        }
        // Down
        if self.y > 0 {
            neighbors.push(TilePoint::new(self.x, self.y - 1));
        }
        // Right
        if self.x < CHUNK_SIZE - 1 {
            neighbors.push(TilePoint::new(self.x + 1, self.y));
        }
        // Left
        if self.x > 0 {
            neighbors.push(TilePoint::new(self.x - 1, self.y));
        }

        neighbors
    }
}

/// A connected region of wall tiles
#[derive(Debug, Clone)]
pub struct WallRegion {
    pub tiles: HashSet<TilePoint>,
    pub boundary_polylines: Vec<Vec<Vec2>>,
}

/// Find all connected wall regions in a chunk using flood fill
pub fn find_wall_regions(tiles: &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]) -> Vec<WallRegion> {
    let mut visited = HashSet::new();
    let mut regions = Vec::new();

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let point = TilePoint::new(x, y);

            if !visited.contains(&point) && tiles[y as usize][x as usize] == TileType::Wall {
                // Found an unvisited wall tile, start flood fill
                let region_tiles = flood_fill_region(tiles, point, &mut visited);
                if !region_tiles.is_empty() {
                    let boundary_polylines = trace_region_boundaries(&region_tiles);
                    regions.push(WallRegion {
                        tiles: region_tiles,
                        boundary_polylines,
                    });
                }
            }
        }
    }

    regions
}

/// Flood fill starting from a wall tile to find all connected wall tiles
fn flood_fill_region(
    tiles: &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    start: TilePoint,
    visited: &mut HashSet<TilePoint>
) -> HashSet<TilePoint> {
    let mut region = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(start);
    visited.insert(start);
    region.insert(start);

    while let Some(current) = queue.pop_front() {
        for neighbor in current.neighbors() {
            if !visited.contains(&neighbor)
                && tiles[neighbor.y as usize][neighbor.x as usize] == TileType::Wall {
                visited.insert(neighbor);
                region.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    region
}

/// Trace the boundaries of a wall region to create polylines
/// This finds both outer boundaries and inner boundaries (holes)
fn trace_region_boundaries(region: &HashSet<TilePoint>) -> Vec<Vec<Vec2>> {
    let mut boundaries = Vec::new();

    if region.is_empty() {
        return boundaries;
    }

    // Find outer boundary
    let outer_boundary = trace_contour_boundary(region);
    if outer_boundary.len() >= 3 {
        boundaries.push(outer_boundary);
    }

    // Find holes (inner boundaries)
    let holes = find_holes_in_region(region);
    for hole in holes {
        // For holes, we need to trace the boundary of the wall around the hole
        let hole_boundary = trace_hole_boundary(&hole, region);
        if hole_boundary.len() >= 3 {
            boundaries.push(hole_boundary);
        }
    }

    boundaries
}

/// Find holes (non-wall regions completely enclosed by walls) within a wall region
fn find_holes_in_region(region: &HashSet<TilePoint>) -> Vec<HashSet<TilePoint>> {
    let mut holes = Vec::new();
    
    if region.is_empty() {
        return holes;
    }
    
    // Find the bounding box of the region
    let min_x = region.iter().map(|p| p.x).min().unwrap();
    let min_y = region.iter().map(|p| p.y).min().unwrap();
    let max_x = region.iter().map(|p| p.x).max().unwrap();
    let max_y = region.iter().map(|p| p.y).max().unwrap();
    
    let mut visited_empty = HashSet::new();
    
    // Look for empty spaces within the bounding box
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let point = TilePoint::new(x, y);
            
            // If this point is empty (not a wall) and not visited
            if !region.contains(&point) && !visited_empty.contains(&point) {
                // Flood fill to find this empty region
                let empty_region = flood_fill_empty_region(&point, region, &mut visited_empty, min_x, min_y, max_x, max_y);
                
                // Check if this empty region is completely enclosed (a hole)
                if is_enclosed_hole(&empty_region, region, min_x, min_y, max_x, max_y) {
                    holes.push(empty_region);
                }
            }
        }
    }
    
    holes
}

/// Flood fill an empty region to find all connected non-wall tiles
fn flood_fill_empty_region(
    start: &TilePoint,
    walls: &HashSet<TilePoint>,
    visited: &mut HashSet<TilePoint>,
    min_x: u32, min_y: u32, max_x: u32, max_y: u32
) -> HashSet<TilePoint> {
    let mut region = HashSet::new();
    let mut queue = VecDeque::new();
    
    queue.push_back(*start);
    visited.insert(*start);
    region.insert(*start);
    
    while let Some(current) = queue.pop_front() {
        // Check 4-directional neighbors
        let neighbors = [
            (current.x.wrapping_sub(1), current.y),
            (current.x + 1, current.y),
            (current.x, current.y.wrapping_sub(1)),
            (current.x, current.y + 1),
        ];
        
        for (nx, ny) in neighbors {
            // Check bounds
            if nx >= min_x && nx <= max_x && ny >= min_y && ny <= max_y {
                let neighbor = TilePoint::new(nx, ny);
                
                if !visited.contains(&neighbor) && !walls.contains(&neighbor) {
                    visited.insert(neighbor);
                    region.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    region
}

/// Check if an empty region is completely enclosed by walls (making it a hole)
fn is_enclosed_hole(
    empty_region: &HashSet<TilePoint>,
    _walls: &HashSet<TilePoint>,
    min_x: u32, min_y: u32, max_x: u32, max_y: u32
) -> bool {
    // A region is a hole if none of its points touch the boundary of the bounding box
    for point in empty_region {
        if point.x == min_x || point.x == max_x || point.y == min_y || point.y == max_y {
            return false; // Touches boundary, not a hole
        }
    }
    true // Completely enclosed
}

/// Trace the boundary around a hole (empty region within walls)
fn trace_hole_boundary(hole: &HashSet<TilePoint>, walls: &HashSet<TilePoint>) -> Vec<Vec2> {
    if hole.is_empty() {
        return Vec::new();
    }

    // Find all boundary edges around the hole
    let mut boundary_edges = Vec::new();
    
    for &empty_tile in hole {
        let x = empty_tile.x as f32;
        let y = empty_tile.y as f32;
        
        // Check each edge of this empty tile to see if it borders a wall
        // North edge (top of empty tile)
        let north_neighbor = TilePoint::new(empty_tile.x, empty_tile.y + 1);
        if walls.contains(&north_neighbor) {
            boundary_edges.push((
                Vec2::new(x + 0.5, y + 0.5),      // Top-right corner (reversed for hole)
                Vec2::new(x - 0.5, y + 0.5),      // Top-left corner
            ));
        }
        
        // East edge (right of empty tile)
        let east_neighbor = TilePoint::new(empty_tile.x + 1, empty_tile.y);
        if walls.contains(&east_neighbor) {
            boundary_edges.push((
                Vec2::new(x + 0.5, y - 0.5),      // Bottom-right corner (reversed for hole)
                Vec2::new(x + 0.5, y + 0.5),      // Top-right corner
            ));
        }
        
        // South edge (bottom of empty tile)
        if empty_tile.y > 0 {
            let south_neighbor = TilePoint::new(empty_tile.x, empty_tile.y - 1);
            if walls.contains(&south_neighbor) {
                boundary_edges.push((
                    Vec2::new(x - 0.5, y - 0.5),  // Bottom-left corner (reversed for hole)
                    Vec2::new(x + 0.5, y - 0.5),  // Bottom-right corner
                ));
            }
        }
        
        // West edge (left of empty tile)
        if empty_tile.x > 0 {
            let west_neighbor = TilePoint::new(empty_tile.x - 1, empty_tile.y);
            if walls.contains(&west_neighbor) {
                boundary_edges.push((
                    Vec2::new(x - 0.5, y + 0.5),  // Top-left corner (reversed for hole)
                    Vec2::new(x - 0.5, y - 0.5),  // Bottom-left corner  
                ));
            }
        }
    }
    
    // Connect these edges to form the hole boundary
    trace_perimeter_from_edges(boundary_edges)
}

/// Trace the exact boundary of a wall region by following the contour
/// This creates a polyline that traces around the perimeter in order
fn trace_contour_boundary(region: &HashSet<TilePoint>) -> Vec<Vec2> {
    if region.is_empty() {
        return Vec::new();
    }

    // Find all boundary edges - edges that separate wall tiles from non-wall tiles
    let mut boundary_edges = Vec::new();

    for &tile in region {
        let x = tile.x as f32;
        let y = tile.y as f32;

        // Check each edge of this tile
        // North edge (top of tile)
        let north_neighbor = TilePoint::new(tile.x, tile.y + 1);
        if !region.contains(&north_neighbor) {
            boundary_edges.push((
                Vec2::new(x - 0.5, y + 0.5),      // Top-left corner
                Vec2::new(x + 0.5, y + 0.5),      // Top-right corner
            ));
        }

        // East edge (right of tile)
        let east_neighbor = TilePoint::new(tile.x + 1, tile.y);
        if !region.contains(&east_neighbor) {
            boundary_edges.push((
                Vec2::new(x + 0.5, y + 0.5),      // Top-right corner
                Vec2::new(x + 0.5, y - 0.5),      // Bottom-right corner
            ));
        }

        // South edge (bottom of tile)
        if tile.y > 0 {
            let south_neighbor = TilePoint::new(tile.x, tile.y - 1);
            if !region.contains(&south_neighbor) {
                boundary_edges.push((
                    Vec2::new(x + 0.5, y - 0.5),  // Bottom-right corner
                    Vec2::new(x - 0.5, y - 0.5),  // Bottom-left corner
                ));
            }
        } else {
            // At y=0, there's always a boundary to the south
            boundary_edges.push((
                Vec2::new(x + 0.5, y - 0.5),  // Bottom-right corner
                Vec2::new(x - 0.5, y - 0.5),  // Bottom-left corner
            ));
        }

        // West edge (left of tile)
        if tile.x > 0 {
            let west_neighbor = TilePoint::new(tile.x - 1, tile.y);
            if !region.contains(&west_neighbor) {
                boundary_edges.push((
                    Vec2::new(x - 0.5, y - 0.5),  // Bottom-left corner
                    Vec2::new(x - 0.5, y + 0.5),  // Top-left corner
                ));
            }
        } else {
            // At x=0, there's always a boundary to the west
            boundary_edges.push((
                Vec2::new(x - 0.5, y - 0.5),  // Bottom-left corner
                Vec2::new(x - 0.5, y + 0.5),  // Top-left corner
            ));
        }
    }

    // Now connect these edges to form the actual perimeter
    trace_perimeter_from_edges(boundary_edges)
}

/// Connect boundary edges to form a coherent perimeter
fn trace_perimeter_from_edges(edges: Vec<(Vec2, Vec2)>) -> Vec<Vec2> {
    if edges.is_empty() {
        return Vec::new();
    }

    let mut perimeter = Vec::new();
    let mut remaining_edges = edges;

    // Start with the first edge
    let current_edge = remaining_edges.remove(0);
    perimeter.push(current_edge.0);
    perimeter.push(current_edge.1);

    // Keep connecting edges until we can't find more connections
    while !remaining_edges.is_empty() {
        let current_end = *perimeter.last().unwrap();
        let mut found_connection = false;

        // Look for an edge that starts where we ended
        for (i, &(start, end)) in remaining_edges.iter().enumerate() {
            if (current_end - start).length() < 0.01 {
                perimeter.push(end);
                remaining_edges.remove(i);
                found_connection = true;
                break;
            }
            // Also try reversed edge
            else if (current_end - end).length() < 0.01 {
                perimeter.push(start);
                remaining_edges.remove(i);
                found_connection = true;
                break;
            }
        }

        if !found_connection {
            break;
        }
    }

    // Close the polygon if we have enough points
    if perimeter.len() >= 3 {
        if let (Some(first), Some(last)) = (perimeter.first(), perimeter.last()) {
            if (first - last).length() > 0.01 {
                perimeter.push(*first);
            }
        }
    }

    perimeter
}

// Note: The detailed edge-based boundary tracing has been simplified
// to use bounding box approach for now. A full marching squares implementation
// would be more complex but could be added later for precise boundaries.

/// Convert tile-space polylines to world-space coordinates for colliders
pub fn polylines_to_world_space(polylines: &[Vec<Vec2>], chunk_world_pos: Vec2, tile_size: f32) -> Vec<Vec<Vec2>> {
    polylines.iter().map(|polyline| {
        polyline.iter().map(|&point| {
            // Convert from tile coordinates to world coordinates
            // Tile coordinates are relative to chunk, world coordinates are absolute
            let tile_world_offset = Vec2::new(
                (point.x - CHUNK_SIZE as f32 * 0.5) * tile_size,
                (point.y - CHUNK_SIZE as f32 * 0.5) * tile_size,
            );
            chunk_world_pos + tile_world_offset
        }).collect()
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_point_neighbors() {
        let point = TilePoint::new(5, 5);
        let neighbors = point.neighbors();
        assert_eq!(neighbors.len(), 4);
        assert!(neighbors.contains(&TilePoint::new(5, 6))); // Up
        assert!(neighbors.contains(&TilePoint::new(5, 4))); // Down
        assert!(neighbors.contains(&TilePoint::new(6, 5))); // Right
        assert!(neighbors.contains(&TilePoint::new(4, 5))); // Left
    }

    #[test]
    fn test_corner_tile_neighbors() {
        // Test corner tile (0,0) has only 2 neighbors
        let point = TilePoint::new(0, 0);
        let neighbors = point.neighbors();
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&TilePoint::new(0, 1))); // Up
        assert!(neighbors.contains(&TilePoint::new(1, 0))); // Right
    }

    #[test]
    fn test_simple_wall_region() {
        // Create a simple 2x2 wall region
        let mut tiles = [[TileType::Floor; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        tiles[0][0] = TileType::Wall;
        tiles[0][1] = TileType::Wall;
        tiles[1][0] = TileType::Wall;
        tiles[1][1] = TileType::Wall;

        let regions = find_wall_regions(&tiles);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].tiles.len(), 4);
    }
}
