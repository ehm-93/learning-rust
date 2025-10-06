use bevy::prelude::*;

use super::tiles::*;
use super::resources::*;

/// System to load tilemap texture atlas on startup
pub fn load_tilemap_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load the tile texture atlas
    let texture_handle: Handle<Image> = asset_server.load("sprites/tiles.png");

    let tilemap_data = TilemapData {
        tiles: generate_test_tilemap(),
        texture_handle,
        tilemap_entity: None,
    };

    commands.insert_resource(tilemap_data);
}

// Tilemap creation is now handled directly in individual scene setup functions

/// System to clean up tilemap entities (useful for scene transitions)
pub fn cleanup_tilemap(
    mut commands: Commands,
    mut tilemap_data: ResMut<TilemapData>,
    tilemap_query: Query<Entity, With<GameTilemap>>,
) {
    for entity in tilemap_query.iter() {
        commands.entity(entity).despawn();
    }
    tilemap_data.tilemap_entity = None;
}
