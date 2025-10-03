use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::*;
use super::collision;
use crate::world::scenes::dungeon::resources::DungeonState;
use crate::world::tiles::{TileType, TILE_SIZE};
use crate::player::Player;

/// System to manage chunk loading around the player
pub fn manage_chunk_loading(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    level: Res<DungeonState>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return; // No player found
    };

    let player_pos = player_transform.translation.truncate();

    // Calculate required chunks (7x7 grid around player for stress testing)
    let required_chunks = ChunkManager::calculate_required_chunks(player_pos, 5);
    let texture_handle = chunk_manager.texture_handle.clone();

    // Load any missing chunks
    for chunk_coord in required_chunks {
        let needs_spawning = {
            let chunk = chunk_manager.get_or_create_chunk(chunk_coord, &level.macro_map);
            chunk.parent_entity.is_none()
        };

        // Spawn chunk if not already spawned
        if needs_spawning {
            let chunk = chunk_manager.get_chunk(chunk_coord).unwrap();
            let parent_entity = spawn_chunk_tilemap(&mut commands, &texture_handle, chunk);

            // Update the chunk with the parent entity
            let chunk = chunk_manager.get_chunk_mut(chunk_coord).unwrap();
            chunk.parent_entity = Some(parent_entity);
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

    // Calculate chunks to unload (beyond distance 5 - buffer beyond 7x7 load area for stress testing)
    let chunks_to_unload = chunk_manager.calculate_chunks_to_unload(player_pos, 10);

    // Unload distant chunks
    for chunk_coord in chunks_to_unload {
        chunk_manager.unload_chunk(chunk_coord, &mut commands);
    }
}

/// Helper function to spawn a tilemap for a chunk with proper hierarchy
fn spawn_chunk_tilemap(
    commands: &mut Commands,
    texture_handle: &Handle<Image>,
    chunk: &Chunk,
) -> Entity {
    let map_size = TilemapSize {
        x: CHUNK_SIZE,
        y: CHUNK_SIZE,
    };

    // Calculate world position for this chunk
    let chunk_world_pos = chunk_coord_to_world_pos(chunk.position);
    let half_chunk_size = (CHUNK_SIZE as f32 * TILE_SIZE) * 0.5;

    // Create the parent entity for this chunk
    let parent_entity = commands.spawn((
        ChunkParent {
            chunk_coord: chunk.position,
        },
        Transform::from_translation(Vec3::new(
            chunk_world_pos.x,
            chunk_world_pos.y,
            0.0,
        )),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    // Spawn the tilemap as a child of the parent
    let tilemap_entity = commands.spawn_empty().id();
    commands.entity(parent_entity).add_child(tilemap_entity);

    let mut tile_storage = TileStorage::empty(map_size);

    // Calculate transform for the tilemap (relative to parent)
    let tilemap_transform = Transform::from_translation(Vec3::new(
        -half_chunk_size,
        -half_chunk_size,
        -1.0,
    ));

    // Create tiles for the chunk
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
            }

            let tile_entity = tile_cmd.id();
            commands.entity(tilemap_entity).add_child(tile_entity);
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Create efficient polygon colliders using flood fill and boundary tracing
    let wall_regions = collision::find_wall_regions(&chunk.tiles);

    for region in wall_regions {
        for boundary_polyline in region.boundary_polylines {
            if boundary_polyline.len() >= 3 {
                // Convert tile-space polyline to world-space coordinates
                let world_polyline = collision::polylines_to_world_space(
                    &[boundary_polyline],
                    Vec2::new(0.0, 0.0), // Offset handled by transform
                    TILE_SIZE
                );

                if let Some(world_points) = world_polyline.first() {
                    // Create a polyline collider from the boundary points
                    let points: Vec<Vec2> = world_points.clone();

                    // Create indices for the polyline (consecutive vertex pairs)
                    let mut indices = Vec::new();
                    for i in 0..(points.len().saturating_sub(1)) {
                        indices.push([i as u32, (i + 1) as u32]);
                    }
                    // Close the loop if we have enough points
                    if points.len() >= 3 {
                        indices.push([points.len() as u32 - 1, 0]);
                    }

                    let collider = bevy_rapier2d::prelude::Collider::polyline(points, Some(indices));

                    let wall_collider = commands.spawn((
                        crate::world::tiles::WallTile,
                        collider,
                        bevy_rapier2d::prelude::RigidBody::Fixed,
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Hidden, // Invisible collider - visual handled by tile
                        ChunkTilemap { chunk_coord: chunk.position }, // Tag for cleanup
                    )).id();

                    commands.entity(parent_entity).add_child(wall_collider);
                }
            }
        }
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

    info!("Spawned chunk {:?} at world pos {:?} with parent entity {:?}", chunk.position, chunk_world_pos, parent_entity);
    parent_entity
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
