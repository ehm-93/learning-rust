//! Chunk system for procedural terrain generation
//!
//! This module handles the spatial partitioning of the world into NxN tile chunks,
//! dynamic loading/unloading based on player position, and chunk-based tile generation.
//! Supports async chunk loading to keep the render pipeline smooth.

pub mod systems;

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::world::constants::{METERS_PER_CHUNK, TILES_PER_METER};

/// State to control whether chunking systems are active
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ChunkingState {
    /// Chunking systems are disabled (static world)
    #[default]
    Disabled,
    /// Chunking systems are active (dynamic chunk loading)
    Enabled,
}

/// Size of a single chunk in tiles (calculated from world constants)
pub const CHUNK_SIZE: u32 = (METERS_PER_CHUNK * TILES_PER_METER) as u32;

/// Chunk coordinate type
pub type ChunkCoord = IVec2;

/// Convert world position to chunk coordinates
pub fn world_pos_to_chunk_coord(world_pos: Vec2) -> ChunkCoord {
    // Each chunk is METERS_PER_CHUNK meters, each meter has TILES_PER_METER tiles
    let chunk_size_world = CHUNK_SIZE as f32 * crate::world::tiles::TILE_SIZE;
    IVec2::new(
        (world_pos.x / chunk_size_world).floor() as i32,
        (world_pos.y / chunk_size_world).floor() as i32,
    )
}

/// Convert chunk coordinates to world position (center of chunk)
pub fn chunk_coord_to_world_pos(chunk_coord: ChunkCoord) -> Vec2 {
    let chunk_size_world = CHUNK_SIZE as f32 * crate::world::tiles::TILE_SIZE;
    Vec2::new(
        chunk_coord.x as f32 * chunk_size_world + chunk_size_world * 0.5,
        chunk_coord.y as f32 * chunk_size_world + chunk_size_world * 0.5,
    )
}

// ===== Event-Driven Chunk Management Components =====

/// Component attached to entities that require chunks to be loaded around them
#[derive(Component, Debug, Clone)]
pub struct ChunkLoader {
    /// Load chunks within this Manhattan distance (forms a square region)
    pub radius: i32,
    /// Unload chunks beyond this distance (must be >= radius)
    pub unload_radius: i32,
    /// Optional preload distance for background loading (must be >= radius, <= unload_radius)
    pub preload_radius: Option<i32>,
}

impl ChunkLoader {
    /// Create a new chunk loader with basic radius settings
    pub fn new(radius: i32) -> Self {
        Self {
            radius,
            unload_radius: radius + 2, // Add buffer to prevent thrashing
            preload_radius: None,
        }
    }

    /// Create a chunk loader with custom unload radius
    pub fn with_unload_radius(radius: i32, unload_radius: i32) -> Self {
        Self {
            radius,
            unload_radius: unload_radius.max(radius),
            preload_radius: None,
        }
    }

    /// Create a chunk loader with preload radius
    pub fn with_preload(radius: i32, unload_radius: i32, preload_radius: i32) -> Self {
        Self {
            radius,
            unload_radius: unload_radius.max(radius),
            preload_radius: Some(preload_radius.max(radius).min(unload_radius)),
        }
    }
}

/// Event published when a chunk needs to be loaded
#[derive(Event, Debug, Clone)]
pub struct LoadChunk {
    /// Which chunk to load
    pub pos: ChunkCoord,
    /// Center of chunk in world space (convenience field)
    pub world_pos: Vec2,
    /// List of loader entities requiring this chunk
    pub loaded_for: Vec<Entity>,
}

/// Event published when a chunk should be loaded in the background (non-critical)
#[derive(Event, Debug, Clone)]
pub struct PreloadChunk {
    /// Which chunk to preload
    pub pos: ChunkCoord,
    /// Center of chunk in world space (convenience field)
    pub world_pos: Vec2,
    /// List of loader entities requesting preload
    pub loaded_for: Vec<Entity>,
}

/// Event published when NO loaders require a chunk anymore
#[derive(Event, Debug, Clone)]
pub struct UnloadChunk {
    /// Which chunk to unload
    pub pos: ChunkCoord,
    /// Center of chunk in world space (convenience field)
    pub world_pos: Vec2,
}

/// Resource that tracks which chunks are currently required by which loaders
#[derive(Resource, Default)]
pub struct ChunkRegistry {
    /// Maps chunk to set of loader entities requiring it
    active_chunks: HashMap<ChunkCoord, HashSet<Entity>>,
}

impl ChunkRegistry {
    /// Get the number of entities requiring a chunk
    pub fn get_refcount(&self, chunk_coord: ChunkCoord) -> usize {
        self.active_chunks.get(&chunk_coord).map_or(0, |set| set.len())
    }

    /// Get all currently active chunks
    pub fn active_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.active_chunks.keys().copied()
    }

    /// Get the entities requiring a specific chunk
    pub fn get_loaders_for_chunk(&self, chunk_coord: ChunkCoord) -> Vec<Entity> {
        self.active_chunks.get(&chunk_coord).map_or(Vec::new(), |set| set.iter().copied().collect())
    }
}

/// Plugin for chunk management systems
pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize chunking state
            .init_state::<ChunkingState>()
            // Initialize event-driven chunk registry
            .init_resource::<ChunkRegistry>()
            // Add chunk events
            .add_event::<LoadChunk>()
            .add_event::<PreloadChunk>()
            .add_event::<UnloadChunk>()
            // Add event-driven chunk management system (runs first to publish events)
            .add_systems(
                Update,
                systems::track_chunk_loaders.run_if(in_state(ChunkingState::Enabled))
            )
            .add_systems(
                OnEnter(ChunkingState::Disabled),
                systems::unload_all_chunks
            );
    }
}
