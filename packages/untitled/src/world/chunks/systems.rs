use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::*;
use crate::world::tiles::{TileType, TILE_SIZE};
use crate::player::Player;

/// System to manage chunk loading around the player
pub fn manage_chunk_loading(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return; // No player found
    };

    let player_pos = player_transform.translation.truncate();

    // Calculate required chunks (5x5 grid around player)
    let required_chunks = ChunkManager::calculate_required_chunks(player_pos, 2); // radius 2 = 5x5 grid
    let texture_handle = chunk_manager.texture_handle.clone();

    // Load any missing chunks
    for chunk_coord in required_chunks {
        let needs_spawning = {
            let chunk = chunk_manager.get_or_create_chunk(chunk_coord);
            chunk.tilemap_entity.is_none()
        };

        // Spawn tilemap if not already spawned
        if needs_spawning {
            let chunk = chunk_manager.get_chunk(chunk_coord).unwrap();
            let tilemap_entity = spawn_chunk_tilemap(&mut commands, &texture_handle, chunk);

            // Update the chunk with the tilemap entity
            let chunk = chunk_manager.get_chunk_mut(chunk_coord).unwrap();
            chunk.tilemap_entity = Some(tilemap_entity);
        }
    }
}

/// System to unload chunks that are too far from the player
pub fn manage_chunk_unloading(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return; // No player found
    };

    let player_pos = player_transform.translation.truncate();

    // Calculate chunks to unload (beyond distance 7)
    let chunks_to_unload = chunk_manager.calculate_chunks_to_unload(player_pos, 7);

    // Unload distant chunks
    for chunk_coord in chunks_to_unload {
        chunk_manager.unload_chunk(chunk_coord, &mut commands);
    }
}

/// Helper function to spawn a tilemap for a chunk
fn spawn_chunk_tilemap(
    commands: &mut Commands,
    texture_handle: &Handle<Image>,
    chunk: &Chunk,
) -> Entity {
    let map_size = TilemapSize {
        x: CHUNK_SIZE,
        y: CHUNK_SIZE,
    };

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Calculate world position for this chunk
    let chunk_world_pos = chunk_coord_to_world_pos(chunk.position);

    // Calculate transform for the tilemap
    // Position tilemap so its bottom-left corner aligns with chunk bottom-left
    let half_chunk_size = (CHUNK_SIZE as f32 * TILE_SIZE) * 0.5;
    let tilemap_transform = Transform::from_translation(Vec3::new(
        chunk_world_pos.x - half_chunk_size,
        chunk_world_pos.y - half_chunk_size,
        -1.0,
    ));

    // Create tiles for the chunk and collect wall positions
    let mut wall_positions = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let tile_pos = TilePos { x, y };
            let tile_type = chunk.tiles[y as usize][x as usize];

            let texture_index = match tile_type {
                TileType::Floor => 0,
                TileType::Wall => 1,
            };

            let mut tile_cmd = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(texture_index),
                ..Default::default()
            });

            if tile_type == TileType::Wall {
                tile_cmd.insert(crate::world::tiles::WallTile);
                wall_positions.push((x, y));
            }

            let tile_entity = tile_cmd.id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Now spawn separate collider entities for each wall
    // Position at bottom-left corner of each tile (like cathedral system)
    for (x, y) in wall_positions {
        let tile_world_x = chunk_world_pos.x - half_chunk_size + x as f32 * TILE_SIZE;
        let tile_world_y = chunk_world_pos.y - half_chunk_size + y as f32 * TILE_SIZE;

        commands.spawn((
            crate::world::tiles::WallTile,
            bevy_rapier2d::prelude::Collider::cuboid(TILE_SIZE * 0.5, TILE_SIZE * 0.5),
            bevy_rapier2d::prelude::RigidBody::Fixed,
            Transform::from_translation(Vec3::new(tile_world_x, tile_world_y, 0.0)),
            Visibility::Hidden, // Invisible collider - visual handled by tile
            ChunkTilemap { chunk_coord: chunk.position }, // Tag for cleanup
        ));
    }

    // Configure the tilemap
    commands
        .entity(tilemap_entity)
        .insert((
            TilemapBundle {
                grid_size: TilemapGridSize { x: TILE_SIZE, y: TILE_SIZE },
                size: map_size,
                storage: tile_storage,
                texture: TilemapTexture::Single(texture_handle.clone()),
                tile_size: TilemapTileSize { x: 16.0, y: 16.0 }, // Texture size in pixels
                transform: tilemap_transform,
                ..Default::default()
            },
            ChunkTilemap {
                chunk_coord: chunk.position,
            },
            crate::world::tiles::GameTilemap,
        ));

    info!("Spawned tilemap for chunk {:?} at world pos {:?}", chunk.position, chunk_world_pos);
    tilemap_entity
}

/// System to initialize the ChunkManager resource
pub fn initialize_chunk_manager(
    mut commands: Commands,
    tilemap_data: Res<crate::world::tiles::TilemapData>,
) {
    let chunk_manager = ChunkManager::new(tilemap_data.texture_handle.clone());
    commands.insert_resource(chunk_manager);
    info!("Initialized ChunkManager");
}

/// System to clean up all chunks when chunking is disabled
pub fn cleanup_all_chunks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let chunk_coords: Vec<ChunkCoord> = chunk_manager.loaded_chunks().collect();

    for chunk_coord in chunk_coords {
        chunk_manager.unload_chunk(chunk_coord, &mut commands);
    }

    info!("Cleaned up all chunks - chunking disabled");
}
