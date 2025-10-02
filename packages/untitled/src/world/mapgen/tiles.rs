use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

/// Tile types for the tilemap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Floor = 0,
    Wall = 1,
}

/// Constants for the test tilemap size
pub const TILEMAP_WIDTH: u32 = 32;
pub const TILEMAP_HEIGHT: u32 = 32;
pub const TILE_SIZE: f32 = 16.0;

/// Generate a simple test pattern with walls around the edges and some internal walls
/// This creates a basic maze-like structure for Phase A testing
pub fn generate_test_tilemap() -> [[TileType; TILEMAP_WIDTH as usize]; TILEMAP_HEIGHT as usize] {
    let mut tiles = [[TileType::Floor; TILEMAP_WIDTH as usize]; TILEMAP_HEIGHT as usize];

    for y in 0..TILEMAP_HEIGHT as usize {
        for x in 0..TILEMAP_WIDTH as usize {
            // Border walls - create a boundary around the entire map
            if x == 0 || x == TILEMAP_WIDTH as usize - 1 ||
               y == 0 || y == TILEMAP_HEIGHT as usize - 1 {
                tiles[y][x] = TileType::Wall;
            }
            // Some internal walls for testing navigation
            else if (x == 10 && y > 5 && y < 15) ||  // Vertical wall
                    (y == 10 && x > 15 && x < 25) || // Horizontal wall
                    (x == 5 && y > 20 && y < 28) ||  // Another vertical wall
                    (y == 20 && x > 8 && x < 18) {   // Another horizontal wall
                tiles[y][x] = TileType::Wall;
            }
            // Create a small room in the corner
            else if (x >= 25 && x <= 29 && y >= 25 && y <= 29) &&
                    (x == 25 || x == 29 || y == 25 || y == 29) &&
                    !(x == 27 && y == 25) { // Leave a door
                tiles[y][x] = TileType::Wall;
            }
        }
    }

    tiles
}
