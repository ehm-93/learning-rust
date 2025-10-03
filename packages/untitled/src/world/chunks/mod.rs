//! Chunk system for procedural terrain generation
//!
//! This module handles the spatial partitioning of the world into NxN tile chunks,
//! dynamic loading/unloading based on player position, and chunk-based tile generation.
//! Supports async chunk loading to keep the render pipeline smooth.

pub mod collision;
pub mod systems;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::collections::HashMap;

use crate::world::constants::{MACRO_PX_PER_CHUNK, METERS_PER_CHUNK, TILES_PER_METER};
use crate::world::tiles;

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

/// Task for loading a chunk asynchronously
pub type ChunkLoadTask = Task<ChunkData>;

/// Data generated for a chunk off the main thread
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkData {
    /// Position of this chunk in chunk coordinates
    pub position: ChunkCoord,
    /// 64x64 tile data for this chunk
    pub tiles: [[tiles::TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    /// Pre-computed wall regions for efficient collision generation
    pub wall_regions: Vec<collision::WallRegion>,
}

/// Loading state for chunk management
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkLoadingState {
    /// Chunk is not loaded and not being loaded
    Unloaded,
    /// Chunk is currently being loaded on a background thread
    Loading,
    /// Chunk data is loaded but not yet spawned
    Loaded(ChunkData),
    /// Chunk is fully spawned in the world
    Spawned(Entity), // Parent entity
}

/// A single chunk containing an NxN grid of tiles (size from CHUNK_SIZE)
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Position of this chunk in chunk coordinates
    pub position: ChunkCoord,
    /// NxN tile data for this chunk (size determined by CHUNK_SIZE constant)
    pub tiles: [[tiles::TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    /// Whether this chunk has been modified and needs to be saved
    pub dirty: bool,
    /// The parent entity for this chunk that contains all chunk-related entities (if spawned)
    pub parent_entity: Option<Entity>,
}

impl ChunkData {
    /// Get the tile at the given local coordinates within this chunk
    pub fn get_tile(&self, local_x: u32, local_y: u32) -> Option<tiles::TileType> {
        if local_x < CHUNK_SIZE && local_y < CHUNK_SIZE {
            Some(self.tiles[local_y as usize][local_x as usize])
        } else {
            None
        }
    }
}

/// Resource managing all loaded chunks with async loading support
#[derive(Resource, Default)]
pub struct ChunkManager {
    /// All chunks and their current loading state
    pub chunks: HashMap<ChunkCoord, ChunkLoadingState>,
    /// Active loading tasks that we poll for completion
    pub loading_tasks: HashMap<ChunkCoord, ChunkLoadTask>,
    /// Texture handle for all tilemaps
    pub texture_handle: Handle<Image>,
}

impl ChunkManager {
    /// Create a new chunk manager with the given texture
    pub fn new(texture_handle: Handle<Image>) -> Self {
        Self {
            chunks: HashMap::new(),
            loading_tasks: HashMap::new(),
            texture_handle,
        }
    }

    /// Start loading a chunk asynchronously if not already loaded/loading
    pub fn request_chunk_load(&mut self, chunk_coord: ChunkCoord, macro_map: Vec<Vec<bool>>) {
        if !matches!(self.chunks.get(&chunk_coord), Some(ChunkLoadingState::Unloaded) | None) {
            return; // Already loading or loaded
        }

        info!("Starting async load for chunk {:?}", chunk_coord);

        // Spawn async task to generate chunk data
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            // Generate tile data
            let tiles = Self::generate_chunk_tiles(chunk_coord, &macro_map);

            // Perform heavy collision computation off main thread
            let wall_regions = collision::find_wall_regions(&tiles);

            ChunkData {
                position: chunk_coord,
                tiles,
                wall_regions,
            }
        });

        self.chunks.insert(chunk_coord, ChunkLoadingState::Loading);
        self.loading_tasks.insert(chunk_coord, task);
    }

    /// Check if a chunk is ready to be spawned (loaded but not yet spawned)
    pub fn is_chunk_ready_to_spawn(&self, chunk_coord: ChunkCoord) -> bool {
        matches!(self.chunks.get(&chunk_coord), Some(ChunkLoadingState::Loaded(_)))
    }

    /// Get chunk data for spawning (moves from Loaded to Spawned state)
    pub fn take_loaded_chunk_data(&mut self, chunk_coord: ChunkCoord) -> Option<ChunkData> {
        if let Some(ChunkLoadingState::Loaded(data)) = self.chunks.remove(&chunk_coord) {
            self.chunks.insert(chunk_coord, ChunkLoadingState::Spawned(Entity::PLACEHOLDER)); // Will be updated with real entity
            Some(data)
        } else {
            None
        }
    }

    /// Mark a chunk as fully spawned with its parent entity
    pub fn mark_chunk_spawned(&mut self, chunk_coord: ChunkCoord, parent_entity: Entity) {
        self.chunks.insert(chunk_coord, ChunkLoadingState::Spawned(parent_entity));
    }

    /// Remove and despawn a chunk
    pub fn unload_chunk(&mut self, chunk_coord: ChunkCoord, commands: &mut Commands) -> bool {
        // Cancel loading task if it exists
        self.loading_tasks.remove(&chunk_coord);

        if let Some(state) = self.chunks.remove(&chunk_coord) {
            match state {
                ChunkLoadingState::Spawned(entity) => {
                    commands.entity(entity).despawn();
                    info!("Unloaded spawned chunk at {:?}", chunk_coord);
                    true
                }
                _ => {
                    info!("Cancelled loading chunk at {:?}", chunk_coord);
                    true
                }
            }
        } else {
            false
        }
    }

    /// Get all loaded/spawned chunk coordinates
    pub fn loaded_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.chunks.iter()
            .filter_map(|(coord, state)| {
                match state {
                    ChunkLoadingState::Loaded(_) | ChunkLoadingState::Spawned(_) => Some(*coord),
                    _ => None,
                }
            })
    }

    /// Get the number of chunks in any state
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get the number of actively loading chunks
    pub fn loading_count(&self) -> usize {
        self.loading_tasks.len()
    }

    /// Generate chunk tiles (extracted for async use)
    fn generate_chunk_tiles(
        position: ChunkCoord,
        macro_map: &Vec<Vec<bool>>
    ) -> [[tiles::TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] {
        let mut tiles = [[tiles::TileType::Floor; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // Macro map is structured as [y][x] (row/column)
        let macro_height = macro_map.len();
        let macro_width = if macro_height > 0 { macro_map[0].len() } else { 0 };

        // If macro map is empty, fill with walls
        if macro_height == 0 || macro_width == 0 {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    tiles[y as usize][x as usize] = tiles::TileType::Wall;
                }
            }
            return tiles;
        }

        // Generate tiles using global coordinate sampling for seamless chunk transitions
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                // Calculate global tile coordinates
                let global_tile_x = position.x as f32 * CHUNK_SIZE as f32 + x as f32;
                let global_tile_y = position.y as f32 * CHUNK_SIZE as f32 + y as f32;

                // Convert to macro space coordinates (continuous)
                let tiles_per_macro = CHUNK_SIZE as f32 / MACRO_PX_PER_CHUNK as f32;
                let macro_x_f = global_tile_x / tiles_per_macro;
                let macro_y_f = global_tile_y / tiles_per_macro;

                // Sample the macro map with smooth interpolation
                let density = Self::sample_macro_density_smooth(
                    macro_map,
                    macro_x_f,
                    macro_y_f,
                    macro_width,
                    macro_height
                );

                // Add very subtle noise for natural variation (reduced intensity)
                let noise_value = Self::noise_2d(global_tile_x, global_tile_y, 0.05);
                let final_density = (density + noise_value * 0.1).clamp(0.0, 1.0);

                // Use a smoother threshold function instead of hard random cutoff
                let smooth_threshold = 0.5 + Self::noise_2d(global_tile_x, global_tile_y, 0.02) * 0.1;

                tiles[y as usize][x as usize] = if final_density > smooth_threshold {
                    tiles::TileType::Wall
                } else {
                    tiles::TileType::Floor
                };
            }
        }

        tiles
    }

    /// Sample macro density with smooth interpolation across chunk boundaries
    fn sample_macro_density_smooth(
        macro_map: &Vec<Vec<bool>>,
        macro_x: f32,
        macro_y: f32,
        macro_width: usize,
        macro_height: usize,
    ) -> f32 {
        // Clamp coordinates to valid range
        let mx = macro_x.max(0.0).min(macro_width as f32 - 0.001);
        let my = macro_y.max(0.0).min(macro_height as f32 - 0.001);

        // Get the four corner points for bilinear interpolation
        let x0 = mx.floor() as usize;
        let y0 = my.floor() as usize;
        let x1 = (x0 + 1).min(macro_width - 1);
        let y1 = (y0 + 1).min(macro_height - 1);

        let fx = mx - x0 as f32;
        let fy = my - y0 as f32;

        // Sample the four corners (convert bool to density: true=open=0.0, false=wall=1.0)
        let d00 = if macro_map[y0][x0] { 0.0 } else { 1.0 };
        let d10 = if macro_map[y0][x1] { 0.0 } else { 1.0 };
        let d01 = if macro_map[y1][x0] { 0.0 } else { 1.0 };
        let d11 = if macro_map[y1][x1] { 0.0 } else { 1.0 };

        // Bilinear interpolation
        let d0 = d00 * (1.0 - fx) + d10 * fx;
        let d1 = d01 * (1.0 - fx) + d11 * fx;

        d0 * (1.0 - fy) + d1 * fy
    }

    /// Simple 2D noise function for micro-variation
    fn noise_2d(x: f32, y: f32, frequency: f32) -> f32 {
        let x = x * frequency;
        let y = y * frequency;

        // Simple hash-based noise
        let hash = ((x as i32).wrapping_mul(73856093) ^ (y as i32).wrapping_mul(19349663)) as u32;
        let normalized = (hash as f32) / (u32::MAX as f32);

        // Convert to -1.0 to 1.0 range
        normalized * 2.0 - 1.0
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
    /// Uses the same square-based distance logic as loading for consistency
    pub fn calculate_chunks_to_unload(&self, world_pos: Vec2, unload_radius: i32) -> Vec<ChunkCoord> {
        let center_chunk = world_pos_to_chunk_coord(world_pos);
        let mut to_unload = Vec::new();

        for &chunk_coord in self.chunks.keys() {
            let distance = (chunk_coord - center_chunk).abs();
            let max_distance = distance.x.max(distance.y);

            // Use the same square-based logic as loading: unload if outside the square
            if max_distance > unload_radius {
                to_unload.push(chunk_coord);
            }
        }

        to_unload
    }

    /// Poll all loading tasks and move completed ones to loaded state
    pub fn poll_loading_tasks(&mut self) {
        let mut completed_tasks = Vec::new();

        // Check each loading task for completion
        for (&chunk_coord, task) in self.loading_tasks.iter_mut() {
            if task.is_finished() {
                completed_tasks.push(chunk_coord);
            }
        }

        // Move completed tasks to loaded state
        for chunk_coord in completed_tasks {
            if let Some(task) = self.loading_tasks.remove(&chunk_coord) {
                // Use Bevy's block_on approach since task is already finished
                let chunk_data = bevy::tasks::block_on(task);
                info!("Chunk {:?} finished loading", chunk_coord);
                self.chunks.insert(chunk_coord, ChunkLoadingState::Loaded(chunk_data));
            }
        }
    }
}

/// Component to mark chunk tilemap entities
#[derive(Component)]
pub struct ChunkTilemap {
    pub chunk_coord: ChunkCoord,
}

/// Component to mark chunk parent entities that hold all chunk-related entities
#[derive(Component)]
pub struct ChunkParent {
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
                systems::poll_chunk_loading_tasks,
                systems::manage_chunk_loading,
                systems::spawn_loaded_chunks.after(systems::poll_chunk_loading_tasks),
                systems::manage_chunk_unloading.after(systems::manage_chunk_loading),
            ).run_if(in_state(ChunkingState::Enabled)))
            // Add cleanup system when entering disabled state
            .add_systems(OnEnter(ChunkingState::Disabled), systems::cleanup_all_chunks);
    }
}
