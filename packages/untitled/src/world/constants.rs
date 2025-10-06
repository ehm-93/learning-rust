pub const MACRO_PX_PER_CHUNK: usize = 4; // Each chunk is 4x4 macro cells
pub const METERS_PER_CHUNK: usize = 8; // Each chunk is 8x8 meters
pub const TILES_PER_METER: usize = 2; // Each meter is 2x2 tiles
pub const PX_PER_TILE: usize = 16; // Each tile is 16x16 pixels
pub const DUNGEON_SIZE_M: usize = 1024;
pub const DUNGEON_SIZE_TILES: usize = DUNGEON_SIZE_M * TILES_PER_METER;
pub const DUNGEON_SIZE_PX: usize = DUNGEON_SIZE_TILES * PX_PER_TILE;
