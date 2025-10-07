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
    let loader_count = loaders.iter().count();
    if loader_count == 0 {
        return; // No loaders, nothing to do
    }

    // Collect all chunks needed by all loaders at different radii
    let mut all_preload_chunks: HashSet<ChunkCoord> = HashSet::new();
    let mut all_load_chunks: HashSet<ChunkCoord> = HashSet::new();
    let mut all_unload_chunks: HashSet<ChunkCoord> = HashSet::new();
    let mut chunk_to_loaders: HashMap<ChunkCoord, Vec<Entity>> = HashMap::new();

    // Process all loaders to determine chunk requirements
    for (entity, transform, loader) in loaders.iter() {
        let world_pos = transform.translation.truncate();
        let chunk_pos = world_pos_to_chunk_coord(world_pos);

        // Calculate chunks at different radii using Manhattan distance
        let load_chunks = calculate_chunks_in_radius(chunk_pos, loader.radius);
        let preload_chunks = if let Some(preload_radius) = loader.preload_radius {
            calculate_chunks_in_radius(chunk_pos, preload_radius)
        } else {
            Vec::new()
        };
        let unload_chunks = calculate_chunks_in_radius(chunk_pos, loader.unload_radius);

        // Add to global sets
        all_load_chunks.extend(load_chunks.iter());
        all_preload_chunks.extend(preload_chunks.iter());
        all_unload_chunks.extend(unload_chunks.iter());

        // Track which loaders need each chunk
        for chunk_coord in &load_chunks {
            chunk_to_loaders.entry(*chunk_coord).or_insert_with(Vec::new).push(entity);
        }
        for chunk_coord in &preload_chunks {
            chunk_to_loaders.entry(*chunk_coord).or_insert_with(Vec::new).push(entity);
        }
        for chunk_coord in &unload_chunks {
            chunk_to_loaders.entry(*chunk_coord).or_insert_with(Vec::new).push(entity);
        }
    }

    // Step 1: Preload events for chunks in preload radius not in registry
    for chunk_coord in &all_preload_chunks {
        if !registry.active_chunks.contains_key(chunk_coord) && !all_load_chunks.contains(chunk_coord) {
            let loader_entities = chunk_to_loaders.get(chunk_coord).cloned().unwrap_or_default();
            preload_events.write(PreloadChunk {
                pos: *chunk_coord,
                world_pos: chunk_coord_to_world_pos(*chunk_coord),
                loaded_for: loader_entities,
            });
        }
    }

    // Step 2: Load events for chunks in load radius not in registry, then add them to registry
    for chunk_coord in &all_load_chunks {
        if !registry.active_chunks.contains_key(chunk_coord) {
            let loader_entities = chunk_to_loaders.get(chunk_coord).cloned().unwrap_or_default();
            load_events.write(LoadChunk {
                pos: *chunk_coord,
                world_pos: chunk_coord_to_world_pos(*chunk_coord),
                loaded_for: loader_entities.clone(),
            });

            // Add to registry
            registry.active_chunks.insert(*chunk_coord, loader_entities.into_iter().collect());
        } else {
            // Update the registry with current loaders for this chunk
            if let Some(existing_loaders) = registry.active_chunks.get_mut(chunk_coord) {
                if let Some(new_loaders) = chunk_to_loaders.get(chunk_coord) {
                    existing_loaders.clear();
                    existing_loaders.extend(new_loaders.iter());
                }
            }
        }
    }

    // Step 3: Unload events for registry chunks outside unload radius, then remove them
    let chunks_to_remove: Vec<ChunkCoord> = registry.active_chunks
        .keys()
        .filter(|chunk_coord| !all_unload_chunks.contains(chunk_coord))
        .copied()
        .collect();

    for chunk_coord in chunks_to_remove {
        unload_events.write(UnloadChunk {
            pos: chunk_coord,
            world_pos: chunk_coord_to_world_pos(chunk_coord),
        });
        registry.active_chunks.remove(&chunk_coord);
    }
}

/// Helper function to calculate chunks within a given Manhattan distance radius
fn calculate_chunks_in_radius(center: ChunkCoord, radius: i32) -> Vec<ChunkCoord> {
    let mut chunks = Vec::new();
    for x in (center.x - radius)..=(center.x + radius) {
        let remaining_distance = radius - (x - center.x).abs();
        for y in (center.y - remaining_distance)..=(center.y + remaining_distance) {
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
