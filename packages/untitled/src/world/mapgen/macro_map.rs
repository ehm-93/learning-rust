//! Macro-level terrain representation
//!
//! The MacroMap represents terrain density at a coarse level where each cell
//! corresponds to one chunk (32x32m area containing 64x64 tiles of 0.5m each).
//! Coordinate system: (0,0) is bottom-left corner, edges are unbreakable walls.

use bevy::prelude::*;
use super::{generate_freeform_paths, flood_fill_validate};

/// Default macro map dimensions (can be overridden)
pub const DEFAULT_MACRO_WIDTH: usize = 64;  // 64 chunks = 2048m width
pub const DEFAULT_MACRO_HEIGHT: usize = 64; // 64 chunks = 2048m height

/// Macro-level terrain density map
/// Each cell represents the density of walls in that chunk area (32x32m, 64x64 tiles)
/// false = mostly open (cave), true = mostly solid (walls)
/// Coordinate system: (0,0) = bottom-left, edges are always solid walls
#[derive(Debug, Clone)]
pub struct MacroMap {
    /// Width of the macro map in chunks
    pub width: usize,
    /// Height of the macro map in chunks
    pub height: usize,
    /// Grid of terrain density: grid[y][x] where (0,0) is bottom-left
    /// false = cave areas, true = wall areas
    /// Edges are always true (unbreakable boundary walls)
    pub grid: Vec<Vec<bool>>,
}

impl MacroMap {
    /// Generate a new macro map using the freeform algorithm with default size
    pub fn generate(seed: u64, level_depth: u32) -> Self {
        Self::generate_sized(DEFAULT_MACRO_WIDTH, DEFAULT_MACRO_HEIGHT, seed, level_depth)
    }

    /// Generate a new macro map with specified dimensions
    pub fn generate_sized(width: usize, height: usize, seed: u64, level_depth: u32) -> Self {
        info!("Generating {}x{} macro map for level {} with seed {}", width, height, level_depth, seed);

        // Initialize with all walls
        let mut grid = vec![vec![true; width]; height];

        // Ensure edges are always solid (unbreakable boundary)
        for x in 0..width {
            grid[0][x] = true;                // Bottom edge
            grid[height - 1][x] = true;      // Top edge
        }
        for y in 0..height {
            grid[y][0] = true;               // Left edge
            grid[y][width - 1] = true;       // Right edge
        }

        // Generate random walks to create connected cave system (only in interior)
        generate_freeform_paths(&mut grid, seed, level_depth);

        // Validate connectivity
        if !flood_fill_validate(&grid) {
            warn!("Generated macro map failed connectivity validation!");
        }

        let macro_map = Self { width, height, grid };

        info!("Successfully generated {}x{} macro map", width, height);
        macro_map
    }

    /// Get the density value at the given macro coordinates
    /// Coordinates: (0,0) = bottom-left corner
    /// Returns true if the area should be mostly walls, false for cave
    pub fn get_density(&self, macro_x: usize, macro_y: usize) -> bool {
        if macro_x >= self.width || macro_y >= self.height {
            return true; // Outside bounds = solid walls
        }
        self.grid[macro_y][macro_x]
    }

    /// Get density using signed coordinates (for easier chunk mapping)
    /// Handles negative coordinates and maps them to the grid properly
    pub fn get_density_signed(&self, macro_x: i32, macro_y: i32) -> bool {
        if macro_x < 0 || macro_y < 0 || macro_x >= self.width as i32 || macro_y >= self.height as i32 {
            return true; // Outside bounds = solid walls
        }
        self.grid[macro_y as usize][macro_x as usize]
    }

    /// Sample densities for a chunk's four quadrants
    /// chunk_x, chunk_y: chunk coordinates (can be negative)
    /// Returns [bottom_left, bottom_right, top_left, top_right] densities
    pub fn sample_chunk_densities(&self, chunk_x: i32, chunk_y: i32) -> [f32; 4] {
        // Map chunk coordinates directly to macro coordinates
        // Since each chunk = 1 macro cell, and (0,0) is bottom-left
        let macro_x = chunk_x;
        let macro_y = chunk_y;

        // Sample this macro cell and its neighbors for smooth interpolation
        let bl = if self.get_density_signed(macro_x, macro_y) { 0.8 } else { 0.2 };
        let br = if self.get_density_signed(macro_x + 1, macro_y) { 0.8 } else { 0.2 };
        let tl = if self.get_density_signed(macro_x, macro_y + 1) { 0.8 } else { 0.2 };
        let tr = if self.get_density_signed(macro_x + 1, macro_y + 1) { 0.8 } else { 0.2 };

        [bl, br, tl, tr]
    }

    /// Count the number of open (cave) cells in the macro map
    pub fn count_open_cells(&self) -> usize {
        self.grid.iter()
            .flat_map(|row| row.iter())
            .filter(|&&cell| !cell)
            .count()
    }

    /// Get the center position for spawning the player
    /// Returns macro coordinates (x, y) where (0,0) is bottom-left
    pub fn get_spawn_position(&self) -> (usize, usize) {
        // Start from center and find nearest open cell
        let center_x = self.width / 2;
        let center_y = self.height / 2;

        // Simple spiral search for nearest open cell
        let max_radius = (self.width.max(self.height) / 2) as i32;
        for radius in 0..max_radius {
            for dx in -(radius as i32)..=(radius as i32) {
                for dy in -(radius as i32)..=(radius as i32) {
                    let x = center_x as i32 + dx;
                    let y = center_y as i32 + dy;

                    if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
                        if !self.grid[y as usize][x as usize] {
                            return (x as usize, y as usize);
                        }
                    }
                }
            }
        }

        // Fallback to center (even if it's solid)
        (center_x, center_y)
    }

    /// Convert macro coordinates to world coordinates (chunk position)
    /// Returns the chunk coordinate corresponding to this macro cell
    pub fn macro_to_chunk_coord(&self, macro_x: usize, macro_y: usize) -> (i32, i32) {
        // Direct mapping: each macro cell = one chunk
        // Adjust for coordinate system where (0,0) chunk is at bottom-left of map
        let chunk_x = macro_x as i32 - (self.width as i32 / 2);
        let chunk_y = macro_y as i32 - (self.height as i32 / 2);
        (chunk_x, chunk_y)
    }

    /// Convert chunk coordinates back to macro coordinates
    pub fn chunk_to_macro_coord(&self, chunk_x: i32, chunk_y: i32) -> (i32, i32) {
        let macro_x = chunk_x + (self.width as i32 / 2);
        let macro_y = chunk_y + (self.height as i32 / 2);
        (macro_x, macro_y)
    }
}
