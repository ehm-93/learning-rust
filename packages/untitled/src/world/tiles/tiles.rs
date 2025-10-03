// Tile definitions and generation functions

/// Tile types for the tilemap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Floor = 0,
    Wall = 1,
}

/// Constants for the cathedral tilemap size
/// Each tile is 0.5m x 0.5m in world units (16 units = 0.5m)
/// 256×256 tiles = 128m×128m world area
pub const TILEMAP_WIDTH: u32 = 256;
pub const TILEMAP_HEIGHT: u32 = 256;
pub const TILE_SIZE: f32 = 16.0; // 16 world units = 0.5m x 0.5m tiles

/// Generate a simple 128m×128m cathedral box
/// Just a large rectangular room with border walls
pub fn generate_test_tilemap() -> [[TileType; TILEMAP_WIDTH as usize]; TILEMAP_HEIGHT as usize] {
    let mut tiles = [[TileType::Floor; TILEMAP_WIDTH as usize]; TILEMAP_HEIGHT as usize];

    let width = TILEMAP_WIDTH as usize;
    let height = TILEMAP_HEIGHT as usize;

    // Create border walls only - simple box
    for x in 0..width {
        tiles[0][x] = TileType::Wall;              // Top wall
        tiles[height - 1][x] = TileType::Wall;     // Bottom wall
    }
    for y in 0..height {
        tiles[y][0] = TileType::Wall;               // Left wall
        tiles[y][width - 1] = TileType::Wall;      // Right wall
    }

    // That's it! Simple 128m×128m box with just border walls
    tiles
}
