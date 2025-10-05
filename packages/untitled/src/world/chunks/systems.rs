use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::*;

// ===== Event-Driven Chunk Systems =====

/// Core system that tracks chunk loader entities and publishes chunk events
pub fn track_chunk_loaders(
    mut registry: ResMut<ChunkRegistry>,
    mut load_events: EventWriter<LoadChunk>,
    mut preload_events: EventWriter<PreloadChunk>,
    mut unload_events: EventWriter<UnloadChunk>,
    loaders: Query<(Entity, &Transform, &ChunkLoader)>,
) {
    // Build new registry from current loaders
    let mut new_active_chunks: HashMap<ChunkCoord, HashSet<Entity>> = HashMap::new();
    let mut chunks_to_load: HashMap<ChunkCoord, Vec<Entity>> = HashMap::new();
    let mut chunks_to_preload: HashMap<ChunkCoord, Vec<Entity>> = HashMap::new();

    // 1. Process all current loaders to determine what chunks are needed
    for (entity, transform, loader) in loaders.iter() {
        let world_pos = transform.translation.truncate();
        let chunk_pos = world_pos_to_chunk_coord(world_pos);

        // Calculate chunks this loader needs
        let critical_chunks = calculate_chunks_in_radius(chunk_pos, loader.radius);
        let preload_chunks = if let Some(preload_radius) = loader.preload_radius {
            calculate_chunks_in_radius(chunk_pos, preload_radius)
        } else {
            Vec::new()
        };

        // Add critical chunks to new registry
        for chunk_coord in critical_chunks {
            new_active_chunks.entry(chunk_coord).or_insert_with(HashSet::new).insert(entity);

            // If this chunk wasn't previously active, mark it for loading
            if !registry.active_chunks.contains_key(&chunk_coord) {
                chunks_to_load.entry(chunk_coord).or_insert_with(Vec::new).push(entity);
            }
        }

        // Add preload chunks (only if not already critical)
        for chunk_coord in preload_chunks {
            if !new_active_chunks.contains_key(&chunk_coord) {
                // If this chunk wasn't previously active, mark it for preloading
                if !registry.active_chunks.contains_key(&chunk_coord) {
                    chunks_to_preload.entry(chunk_coord).or_insert_with(Vec::new).push(entity);
                }
            }
        }
    }

    // 2. Find chunks that need unloading (beyond all loaders' unload radius)
    let chunks_to_unload: Vec<ChunkCoord> = registry.active_chunks.iter()
        .filter_map(|(chunk_coord, _)| {
            // Check if this chunk is beyond all loaders' unload radius
            let should_unload = loaders.iter().all(|(_entity, transform, loader)| {
                let world_pos = transform.translation.truncate();
                let loader_chunk_pos = world_pos_to_chunk_coord(world_pos);
                let distance = (chunk_coord.x - loader_chunk_pos.x).abs().max((chunk_coord.y - loader_chunk_pos.y).abs());
                distance > loader.unload_radius
            });

            if should_unload {
                Some(*chunk_coord)
            } else {
                None
            }
        })
        .collect();

    // 3. Publish events in correct order
    // First: LoadChunk events (critical chunks)
    for (chunk_coord, loader_entities) in chunks_to_load {
        load_events.write(LoadChunk {
            pos: chunk_coord,
            world_pos: chunk_coord_to_world_pos(chunk_coord),
            loaded_for: loader_entities,
        });
    }

    // Second: PreloadChunk events (background chunks)
    for (chunk_coord, loader_entities) in chunks_to_preload {
        preload_events.write(PreloadChunk {
            pos: chunk_coord,
            world_pos: chunk_coord_to_world_pos(chunk_coord),
            loaded_for: loader_entities,
        });
    }

    // Third: UnloadChunk events
    for chunk_coord in chunks_to_unload {
        unload_events.write(UnloadChunk {
            pos: chunk_coord,
            world_pos: chunk_coord_to_world_pos(chunk_coord),
        });
    }

    // 4. Update registry with new state
    registry.active_chunks = new_active_chunks;
}

/// Helper function to calculate chunks within a given radius
fn calculate_chunks_in_radius(center: ChunkCoord, radius: i32) -> Vec<ChunkCoord> {
    let mut chunks = Vec::new();
    for x in (center.x - radius)..=(center.x + radius) {
        for y in (center.y - radius)..=(center.y + radius) {
            chunks.push(ChunkCoord::new(x, y));
        }
    }
    chunks
}

pub fn unload_all_chunks(
    mut registry: ResMut<ChunkRegistry>,
    mut unload_events: EventWriter<UnloadChunk>,
) {
    for (chunk_coord, _) in registry.active_chunks.drain() {
        unload_events.write(UnloadChunk {
            pos: chunk_coord,
            world_pos: chunk_coord_to_world_pos(chunk_coord),
        });
    }
}
