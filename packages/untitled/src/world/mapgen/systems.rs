use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;

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
    info!("Loaded tilemap texture and initialized tilemap data");
}

/// Helper function to spawn tilemap for Cathedral scene
pub fn spawn_cathedral_tilemap(commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let texture_handle: Handle<Image> = asset_server.load("sprites/tiles.png");

    let map_size = TilemapSize { x: TILEMAP_WIDTH as u32, y: TILEMAP_HEIGHT as u32 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Generate the test tilemap pattern
    let tilemap_data = generate_test_tilemap();

    // Store wall positions for collision spawning
    let mut wall_positions = Vec::new();

    // Spawn each tile
    for x in 0..TILEMAP_WIDTH {
        for y in 0..TILEMAP_HEIGHT {
            let tile_pos = TilePos { x: x as u32, y: y as u32 };
            let tile_type = tilemap_data[y as usize][x as usize];

            // Set texture index based on tile type
            let texture_index = match tile_type {
                TileType::Floor => TileTextureIndex(0),  // First tile (dark gray)
                TileType::Wall => TileTextureIndex(1),   // Second tile (light gray)
            };

            let tile_entity = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index,
                ..Default::default()
            }).id();

            // Mark wall tiles and collect positions for collision
            if tile_type == TileType::Wall {
                commands.entity(tile_entity).insert(WallTile);
                wall_positions.push((x, y));
            }

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Now spawn separate collider entities for each wall
    for (x, y) in wall_positions {
        let world_x = x as f32 * TILE_SIZE;
        let world_y = y as f32 * TILE_SIZE;

        commands.spawn((
            WallTile,
            Collider::cuboid(TILE_SIZE / 2.0, TILE_SIZE / 2.0),
            RigidBody::Fixed,
            Transform::from_xyz(world_x, world_y, -1.0),
            Visibility::Hidden, // Invisible collider - visual handled by tile
            crate::world::scenes::cathedral::Cathedral, // Tag for cleanup
        ));
    }

    // Define tilemap properties
    let grid_size = TilemapGridSize { x: TILE_SIZE, y: TILE_SIZE };
    let map_type = TilemapType::Square;
    let tile_size = TilemapTileSize { x: TILE_SIZE, y: TILE_SIZE };

    // Configure the main tilemap entity with proper Z ordering
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: Transform::from_xyz(0.0, 0.0, -1.0), // Behind player
        ..Default::default()
    })
    .insert(GameTilemap)
    .insert(crate::world::scenes::cathedral::Cathedral); // Tag for cleanup

    info!("Successfully spawned Cathedral tilemap with {} tiles", TILEMAP_WIDTH * TILEMAP_HEIGHT);

    tilemap_entity
}

/// System to clean up tilemap entities (useful for scene transitions)
pub fn cleanup_tilemap(
    mut commands: Commands,
    mut tilemap_data: ResMut<TilemapData>,
    tilemap_query: Query<Entity, With<GameTilemap>>,
) {
    for entity in tilemap_query.iter() {
        commands.entity(entity).despawn();
        info!("Despawned tilemap entity");
    }
    tilemap_data.tilemap_entity = None;
}
