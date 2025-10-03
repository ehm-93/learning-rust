//! Chunk system for procedural terrain generation
//!
//! This module handles the spatial partitioning of the world into 64x64 tile chunks,
//! dynamic loading/unloading based on player position, and chunk-based tile generation.

pub mod systems;

use bevy::prelude::*;
use std::collections::HashMap;

use crate::world::tiles::TileType;

/// State to control whether chunking systems are active
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ChunkingState {
    /// Chunking systems are disabled (static world)
    #[default]
    Disabled,
    /// Chunking systems are active (dynamic chunk loading)
    Enabled,
}

/// Size of a single chunk in tiles (64x64 tiles = 32m x 32m at 0.5m/tile)
pub const CHUNK_SIZE: u32 = 64;

/// Chunk coordinate type
pub type ChunkCoord = IVec2;

/// Convert world position to chunk coordinates
pub fn world_pos_to_chunk_coord(world_pos: Vec2) -> ChunkCoord {
    // Each tile is 0.5m (16 units), chunk is 64 tiles = 32m
    let chunk_size_world = CHUNK_SIZE as f32 * 16.0; // 64 * 16 = 1024 units = 32m
    IVec2::new(
        (world_pos.x / chunk_size_world).floor() as i32,
        (world_pos.y / chunk_size_world).floor() as i32,
    )
}

/// Convert chunk coordinates to world position (center of chunk)
pub fn chunk_coord_to_world_pos(chunk_coord: ChunkCoord) -> Vec2 {
    let chunk_size_world = CHUNK_SIZE as f32 * 16.0;
    Vec2::new(
        chunk_coord.x as f32 * chunk_size_world + chunk_size_world * 0.5,
        chunk_coord.y as f32 * chunk_size_world + chunk_size_world * 0.5,
    )
}

/// A single chunk containing a 64x64 grid of tiles
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Position of this chunk in chunk coordinates
    pub position: ChunkCoord,
    /// 64x64 tile data for this chunk
    pub tiles: [[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    /// Whether this chunk has been modified and needs to be saved
    pub dirty: bool,
    /// The tilemap entity for this chunk (if spawned)
    pub tilemap_entity: Option<Entity>,
}

impl Chunk {
    /// Create a new chunk at the given position with default tile generation
    pub fn new(position: ChunkCoord) -> Self {
        Self {
            position,
            tiles: Self::generate_hardcoded_tiles(position),
            dirty: false,
            tilemap_entity: None,
        }
    }

    /// Generate tiles for this chunk (temporary hardcoded implementation)
    /// This will be replaced with proper generation in a later phase
    fn generate_hardcoded_tiles(position: ChunkCoord) -> [[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] {
        let mut tiles = [[TileType::Floor; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // Create a simple pattern based on chunk position
        // Center chunks (0,0) area should be mostly floor
        // Outer chunks should have some walls

        let distance_from_origin = ((position.x.pow(2) + position.y.pow(2)) as f32).sqrt();

        if distance_from_origin > 2.0 {
            // Far chunks: create border walls
            for x in 0..CHUNK_SIZE as usize {
                tiles[0][x] = TileType::Wall; // Top
                tiles[CHUNK_SIZE as usize - 1][x] = TileType::Wall; // Bottom
            }
            for y in 0..CHUNK_SIZE as usize {
                tiles[y][0] = TileType::Wall; // Left
                tiles[y][CHUNK_SIZE as usize - 1] = TileType::Wall; // Right
            }
        } else {
            // Near chunks: mostly floor with some scattered walls
            for x in (10..CHUNK_SIZE as usize).step_by(15) {
                for y in (10..CHUNK_SIZE as usize).step_by(15) {
                    if x < CHUNK_SIZE as usize - 1 && y < CHUNK_SIZE as usize - 1 {
                        tiles[y][x] = TileType::Wall;
                        tiles[y + 1][x] = TileType::Wall;
                        tiles[y][x + 1] = TileType::Wall;
                    }
                }
            }
        }

        tiles
    }

    /// Get the tile at the given local coordinates within this chunk
    pub fn get_tile(&self, local_x: u32, local_y: u32) -> Option<TileType> {
        if local_x < CHUNK_SIZE && local_y < CHUNK_SIZE {
            Some(self.tiles[local_y as usize][local_x as usize])
        } else {
            None
        }
    }

    /// Set the tile at the given local coordinates within this chunk
    pub fn set_tile(&mut self, local_x: u32, local_y: u32, tile_type: TileType) -> bool {
        if local_x < CHUNK_SIZE && local_y < CHUNK_SIZE {
            self.tiles[local_y as usize][local_x as usize] = tile_type;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Mark this chunk as clean (saved to persistence)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

/// Resource managing all loaded chunks
#[derive(Resource, Default)]
pub struct ChunkManager {
    /// All loaded chunks, keyed by their chunk coordinates
    pub chunks: HashMap<ChunkCoord, Chunk>,
    /// Texture handle for all tilemaps
    pub texture_handle: Handle<Image>,
}

impl ChunkManager {
    /// Create a new chunk manager with the given texture
    pub fn new(texture_handle: Handle<Image>) -> Self {
        Self {
            chunks: HashMap::new(),
            texture_handle,
        }
    }

    /// Get a chunk at the given coordinates, loading it if necessary
    pub fn get_or_create_chunk(&mut self, chunk_coord: ChunkCoord) -> &mut Chunk {
        self.chunks.entry(chunk_coord).or_insert_with(|| {
            info!("Creating new chunk at {:?}", chunk_coord);
            Chunk::new(chunk_coord)
        })
    }

    /// Get a reference to a chunk if it exists
    pub fn get_chunk(&self, chunk_coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&chunk_coord)
    }

    /// Get a mutable reference to a chunk if it exists
    pub fn get_chunk_mut(&mut self, chunk_coord: ChunkCoord) -> Option<&mut Chunk> {
        self.chunks.get_mut(&chunk_coord)
    }

    /// Remove and despawn a chunk
    pub fn unload_chunk(&mut self, chunk_coord: ChunkCoord, commands: &mut Commands) -> bool {
        if let Some(chunk) = self.chunks.remove(&chunk_coord) {
            if let Some(entity) = chunk.tilemap_entity {
                commands.entity(entity).despawn();
                info!("Unloaded chunk at {:?}", chunk_coord);
            }
            true
        } else {
            false
        }
    }

    /// Get all loaded chunk coordinates
    pub fn loaded_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.chunks.keys().copied()
    }

    /// Get the number of loaded chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Calculate which chunks should be loaded around a world position
    pub fn calculate_required_chunks(world_pos: Vec2, radius: i32) -> Vec<ChunkCoord> {
        let center_chunk = world_pos_to_chunk_coord(world_pos);
        let mut required = Vec::new();

        for x in -radius..=radius {
            for y in -radius..=radius {
                required.push(ChunkCoord::new(center_chunk.x + x, center_chunk.y + y));
            }
        }

        required
    }

    /// Calculate which chunks should be unloaded (beyond the given radius)
    pub fn calculate_chunks_to_unload(&self, world_pos: Vec2, unload_radius: i32) -> Vec<ChunkCoord> {
        let center_chunk = world_pos_to_chunk_coord(world_pos);
        let mut to_unload = Vec::new();

        for &chunk_coord in self.chunks.keys() {
            let distance = (chunk_coord - center_chunk).abs();
            let max_distance = distance.x.max(distance.y);

            if max_distance > unload_radius {
                to_unload.push(chunk_coord);
            }
        }

        to_unload
    }
}

/// Component to mark chunk tilemap entities
#[derive(Component)]
pub struct ChunkTilemap {
    pub chunk_coord: ChunkCoord,
}

/// Plugin for chunk management systems
pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize chunking state
            .init_state::<ChunkingState>()
            // Initialize chunk manager resource
            .init_resource::<ChunkManager>()
            // Add startup systems for initializing chunk manager
            .add_systems(Startup,
                systems::initialize_chunk_manager
                    .after(crate::world::tiles::systems::load_tilemap_texture)
            )
            // Add chunk management systems (only when chunking is enabled)
            .add_systems(Update, (
                systems::manage_chunk_loading,
                systems::manage_chunk_unloading.after(systems::manage_chunk_loading),
            ).run_if(in_state(ChunkingState::Enabled)))
            // Add cleanup system when entering disabled state
            .add_systems(OnEnter(ChunkingState::Disabled), systems::cleanup_all_chunks);
    }
}
