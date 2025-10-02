use bevy::prelude::*;

use super::tiles::*;

/// Resource to hold tilemap data and state
#[derive(Resource)]
pub struct TilemapData {
    pub tiles: [[TileType; TILEMAP_WIDTH as usize]; TILEMAP_HEIGHT as usize],
    pub texture_handle: Handle<Image>,
    pub tilemap_entity: Option<Entity>,
}

impl Default for TilemapData {
    fn default() -> Self {
        Self {
            tiles: generate_test_tilemap(),
            texture_handle: Handle::default(),
            tilemap_entity: None,
        }
    }
}

/// Component to mark tilemap entities for identification and cleanup
#[derive(Component)]
pub struct GameTilemap;

/// Component to mark wall tiles for collision detection
#[derive(Component)]
pub struct WallTile;
