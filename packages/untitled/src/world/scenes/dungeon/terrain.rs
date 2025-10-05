//! Event-driven terrain chunk management
//!
//! This module handles terrain-specific chunk loading/unloading by subscribing
//! to chunk events from the core chunking system.
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_ecs_tilemap::prelude::*;
use std::collections::HashMap;

use super::collision::*;
use crate::world::chunks::*;
use crate::world::tiles::{TileType, TILE_SIZE};
use crate::world::scenes::dungeon::resources::DungeonState;
use crate::world::constants::MACRO_PX_PER_CHUNK;
use crate::world::PX_PER_TILE;

// === Terrain Generation Constants ===

/// Base density threshold for wall placement (0.5 = 50% probability)
const WALL_DENSITY_THRESHOLD: f32 = 0.5;

/// Lower bound for noise application range (only apply noise near boundaries)
const NOISE_BOUNDARY_MIN: f32 = 0.45;
/// Upper bound for noise application range (only apply noise near boundaries)
const NOISE_BOUNDARY_MAX: f32 = 0.55;

/// Noise frequency - lower values create smoother, less detailed noise
const NOISE_FREQUENCY: f32 = 0.01;
/// Noise amplitude - controls how much the noise affects wall placement
const NOISE_AMPLITUDE: f32 = 0.02;

// === Tilemap Rendering Constants ===

/// Size of each tile texture in pixels (must match sprite sheet)
const TEXTURE_TILE_SIZE: f32 = PX_PER_TILE as f32;
/// Z-depth for tilemap rendering (negative to render behind entities)
const TILEMAP_Z_DEPTH: f32 = -1.0;

// === Macro Map Sampling Constants ===

/// Small epsilon to prevent index out of bounds when sampling macro map
const MACRO_CLAMP_EPSILON: f32 = 1.001;

// === Noise Function Constants ===

/// Hash function X coefficient for pseudo-random noise generation
const NOISE_HASH_X: f32 = 12.9898;
/// Hash function Y coefficient for pseudo-random noise generation
const NOISE_HASH_Y: f32 = 78.233;
/// Hash function multiplier for pseudo-random noise generation
const NOISE_HASH_MULTIPLIER: f32 = 43758.5453;

// === Texture Mapping Constants ===

/// Texture atlas index for floor tiles
const FLOOR_TEXTURE_INDEX: u32 = 0;
/// Texture atlas index for wall tiles
const WALL_TEXTURE_INDEX: u32 = 1;

// === Macro Map Value Mappings ===

/// Density value for floor areas (white pixels in macro map)
const MACRO_FLOOR_VALUE: f32 = 0.0;
/// Density value for wall areas (black pixels in macro map)
const MACRO_WALL_VALUE: f32 = 1.0;

// === Collision Generation Constants ===

/// Minimum number of vertices required to create a valid polyline collider
const MIN_POLYLINE_VERTICES: usize = 3;
/// Factor for calculating half chunk size (0.5 = half)
const CHUNK_HALF_SIZE_FACTOR: f32 = 0.5;

// === Noise Scaling Constants ===

/// Scale factor to convert noise from [0,1] to [-1,1] range
const NOISE_SCALE_FACTOR: f32 = 2.0;
/// Offset to center noise around zero
const NOISE_OFFSET: f32 = 1.0;

/// Terrain-specific chunk state
#[derive(Debug)]
pub enum TerrainChunkState {
    /// Async terrain generation in progress
    Loading { task: Task<ChunkData> },
    /// Spawned in world with entity references
    Loaded {
        entity: Entity,
        tiles: [[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    },
}

/// Resource managing terrain chunks separately from core chunk tracking
#[derive(Resource, Default)]
pub struct TerrainChunks {
    /// Per-chunk terrain state
    chunks: HashMap<ChunkCoord, TerrainChunkState>,
    /// Shared tilemap texture
    texture_handle: Handle<Image>,
}

impl TerrainChunks {
    pub fn new(texture_handle: Handle<Image>) -> Self {
        Self {
            chunks: HashMap::new(),
            texture_handle,
        }
    }

    pub fn get_loaded_count(&self) -> usize {
        self.chunks.len()
    }
}

/// System that listens for LoadChunk events and starts async terrain generation
pub fn handle_chunk_load_events(
    mut terrain_chunks: ResMut<TerrainChunks>,
    mut load_events: EventReader<LoadChunk>,
    mut preload_events: EventReader<PreloadChunk>,
    dungeon_state: Res<DungeonState>,
) {
    // Process critical load events first (higher priority)
    for event in load_events.read() {
        handle_single_load_event(&mut terrain_chunks, event.pos, &dungeon_state);
    }

    // Then process preload events (lower priority)
    for event in preload_events.read() {
        handle_single_load_event(&mut terrain_chunks, event.pos, &dungeon_state);
    }
}

/// Helper function to handle a single chunk load event
fn handle_single_load_event(
    terrain_chunks: &mut TerrainChunks,
    chunk_coord: ChunkCoord,
    dungeon_state: &DungeonState,
) {
    // Check if chunk is already loading or loaded
    if terrain_chunks.chunks.contains_key(&chunk_coord) {
        return; // Already being handled
    }

    info!("Starting terrain generation for chunk {:?}", chunk_coord);

    // Spawn async task to generate terrain data
    let task_pool = AsyncComputeTaskPool::get();
    let macro_map = dungeon_state.macro_map.clone();

    let task = task_pool.spawn(async move {
        // Generate tile data (reuse existing logic from ChunkManager)
        let tiles = generate_chunk_tiles(chunk_coord, &macro_map);

        // Perform heavy collision computation off main thread
        let wall_regions = find_wall_regions(&tiles);

        ChunkData {
            position: chunk_coord,
            tiles,
            wall_regions,
        }
    });

    terrain_chunks.chunks.insert(chunk_coord, TerrainChunkState::Loading { task });
}

/// System that checks async tasks and spawns terrain entities when complete
pub fn poll_terrain_loading_tasks(
    mut commands: Commands,
    mut terrain_chunks: ResMut<TerrainChunks>,
) {
    let mut completed_chunks = Vec::new();

    // Check which tasks have completed
    for (chunk_coord, state) in terrain_chunks.chunks.iter_mut() {
        if let TerrainChunkState::Loading { task } = state {
            if task.is_finished() {
                completed_chunks.push(*chunk_coord);
            }
        }
    }

    // Process completed chunks
    for chunk_coord in completed_chunks {
        if let Some(TerrainChunkState::Loading { task }) = terrain_chunks.chunks.remove(&chunk_coord) {
            match bevy::tasks::block_on(task) {
                chunk_data => {
                    // Spawn the terrain entities
                    let parent_entity = spawn_chunk_tilemap(&mut commands, &terrain_chunks.texture_handle, &chunk_data);

                    // Update state to loaded
                    terrain_chunks.chunks.insert(chunk_coord, TerrainChunkState::Loaded {
                        entity: parent_entity,
                        tiles: chunk_data.tiles,
                    });

                    // Count wall tiles for debugging
                    let wall_count = chunk_data.tiles.iter().flatten().filter(|&&tile| tile == TileType::Wall).count();
                    info!("Completed terrain generation for chunk {:?} with {} wall tiles spawned (entity: {:?})",
                          chunk_coord, wall_count, parent_entity);
                }
            }
        }
    }
}

/// System that listens for UnloadChunk events and despawns terrain
pub fn handle_chunk_unload_events(
    mut commands: Commands,
    mut terrain_chunks: ResMut<TerrainChunks>,
    mut unload_events: EventReader<UnloadChunk>,
) {
    for event in unload_events.read() {
        let chunk_coord = event.pos;

        if let Some(state) = terrain_chunks.chunks.remove(&chunk_coord) {
            match state {
                TerrainChunkState::Loading { task: _ } => {
                    // Task will be dropped automatically, canceling the async work
                    info!("Cancelled terrain loading for chunk {:?}", chunk_coord);
                }
                TerrainChunkState::Loaded { entity, tiles: _ } => {
                    // Despawn the terrain entity tree
                    commands.entity(entity).despawn();
                    info!("Unloaded terrain for chunk {:?}", chunk_coord);
                }
            }
        }
    }
}

/// Helper function to spawn a tilemap for a chunk with proper hierarchy
fn spawn_chunk_tilemap(
    commands: &mut Commands,
    texture_handle: &Handle<Image>,
    chunk_data: &ChunkData,
) -> Entity {
    let map_size = TilemapSize {
        x: CHUNK_SIZE,
        y: CHUNK_SIZE,
    };

    // Calculate world position for this chunk
    let chunk_world_pos = chunk_coord_to_world_pos(chunk_data.position);
    let half_chunk_size = (CHUNK_SIZE as f32 * TILE_SIZE) * CHUNK_HALF_SIZE_FACTOR;

    // Create the parent entity for this chunk
    let parent_entity = commands.spawn((
        ChunkParent {
            chunk_coord: chunk_data.position,
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
        TILEMAP_Z_DEPTH,
    ));

    // Create tiles for the chunk
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let tile_pos = TilePos { x, y };
            let tile_type = chunk_data.tiles[x as usize][y as usize];

            let texture_index = match tile_type {
                TileType::Floor => FLOOR_TEXTURE_INDEX,
                TileType::Wall => WALL_TEXTURE_INDEX,
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

    // Use pre-computed wall regions (calculated async off main thread)
    let wall_regions = &chunk_data.wall_regions;

    for region in wall_regions {
        for boundary_polyline in &region.boundary_polylines {
            if boundary_polyline.len() >= MIN_POLYLINE_VERTICES {
                // Convert tile-space polyline to world-space coordinates
                let world_polyline = polylines_to_world_space(
                    &[boundary_polyline.clone()],
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
                        ChunkTilemap { chunk_coord: chunk_data.position }, // Tag for cleanup
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
                tile_size: TilemapTileSize { x: TEXTURE_TILE_SIZE, y: TEXTURE_TILE_SIZE }, // Texture size in pixels
                transform: tilemap_transform,
                ..Default::default()
            },
            ChunkTilemap {
                chunk_coord: chunk_data.position,
            },
            crate::world::tiles::GameTilemap,
        ));

    info!("Spawned chunk {:?} at world pos {:?} with parent entity {:?}", chunk_data.position, chunk_world_pos, parent_entity);
    parent_entity
}

/// Generate chunk tiles (copied from ChunkManager for async use)
fn generate_chunk_tiles(
    position: ChunkCoord,
    macro_map: &Vec<Vec<bool>>,
) -> [[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] {
    let mut tiles = [[TileType::Floor; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

    for x in 0..CHUNK_SIZE as usize {
        for y in 0..CHUNK_SIZE as usize {
            // Calculate global tile coordinates (matching mod.rs approach)
            let global_tile_x = position.x as f32 * CHUNK_SIZE as f32 + x as f32;
            let global_tile_y = position.y as f32 * CHUNK_SIZE as f32 + y as f32;

            // Convert to macro space coordinates (continuous)
            let tiles_per_macro = CHUNK_SIZE as f32 / MACRO_PX_PER_CHUNK as f32;
            let macro_x_f = global_tile_x / tiles_per_macro;
            let macro_y_f = global_tile_y / tiles_per_macro;

            // Sample the macro map with smooth interpolation
            let density = sample_macro_density_smooth(
                macro_map,
                macro_x_f,
                macro_y_f,
                macro_map[0].len(),
                macro_map.len(),
            );

            // Add minimal noise only very close to the boundary for smoother walls
            let boundary_noise = if density > NOISE_BOUNDARY_MIN && density < NOISE_BOUNDARY_MAX {
                noise_2d(global_tile_x, global_tile_y, NOISE_FREQUENCY) * NOISE_AMPLITUDE
            } else {
                0.0
            };

            let final_threshold = WALL_DENSITY_THRESHOLD + boundary_noise;

            tiles[x][y] = if density > final_threshold {
                TileType::Wall
            } else {
                TileType::Floor
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
    // Clamp to valid range
    let macro_x = macro_x.clamp(0.0, macro_width as f32 - MACRO_CLAMP_EPSILON);
    let macro_y = macro_y.clamp(0.0, macro_height as f32 - MACRO_CLAMP_EPSILON);

    // Get integer and fractional parts
    let x0 = macro_x.floor() as usize;
    let y0 = macro_y.floor() as usize;
    let x1 = (x0 + 1).min(macro_width - 1);
    let y1 = (y0 + 1).min(macro_height - 1);

    let fx = macro_x.fract();
    let fy = macro_y.fract();

    // Sample four corners
    // Note: true = white (floors), false = black (walls) in macro map
    let v00 = if macro_map[y0][x0] { MACRO_FLOOR_VALUE } else { MACRO_WALL_VALUE };
    let v10 = if macro_map[y0][x1] { MACRO_FLOOR_VALUE } else { MACRO_WALL_VALUE };
    let v01 = if macro_map[y1][x0] { MACRO_FLOOR_VALUE } else { MACRO_WALL_VALUE };
    let v11 = if macro_map[y1][x1] { MACRO_FLOOR_VALUE } else { MACRO_WALL_VALUE };

    // Bilinear interpolation
    let v0 = v00 * (1.0 - fx) + v10 * fx;
    let v1 = v01 * (1.0 - fx) + v11 * fx;
    v0 * (1.0 - fy) + v1 * fy
}

/// Simple 2D noise function for micro-variation
fn noise_2d(x: f32, y: f32, frequency: f32) -> f32 {
    let x = x * frequency;
    let y = y * frequency;

    // Simple hash-based noise
    let n = ((x * NOISE_HASH_X + y * NOISE_HASH_Y).sin() * NOISE_HASH_MULTIPLIER).fract();
    n * NOISE_SCALE_FACTOR - NOISE_OFFSET // Scale to [-1, 1]
}

/// System to initialize the TerrainChunks resource
pub fn initialize_terrain_chunks(
    mut commands: Commands,
    tilemap_data: Res<crate::world::tiles::TilemapData>,
) {
    let terrain_chunks = TerrainChunks::new(tilemap_data.texture_handle.clone());
    commands.insert_resource(terrain_chunks);
    info!("Initialized TerrainChunks");
}

/// Plugin for event-driven terrain chunk management
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize terrain chunks resource
            .add_systems(Startup,
                initialize_terrain_chunks
                    .after(crate::world::tiles::systems::load_tilemap_texture)
            )
            // Add terrain chunk management systems (only when chunking is enabled)
            // These run after the core chunk tracking system publishes events
            .add_systems(Update, (
                handle_chunk_load_events
                    .after(crate::world::chunks::systems::track_chunk_loaders),
                poll_terrain_loading_tasks
                    .after(handle_chunk_load_events),
                handle_chunk_unload_events
                    .after(poll_terrain_loading_tasks),
            ).run_if(in_state(ChunkingState::Enabled)));
    }
}

/// Task for loading a chunk asynchronously
type ChunkLoadTask = Task<ChunkData>;

/// Data generated for a chunk off the main thread
#[derive(Debug, Clone, PartialEq)]
struct ChunkData {
    /// Position of this chunk in chunk coordinates
    position: ChunkCoord,
    /// 64x64 tile data for this chunk
    tiles: [[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    /// Pre-computed wall regions for efficient collision generation
    wall_regions: Vec<WallRegion>,
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
struct Chunk {
    /// Position of this chunk in chunk coordinates
    pub position: ChunkCoord,
    /// NxN tile data for this chunk (size determined by CHUNK_SIZE constant)
    pub tiles: [[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    /// Whether this chunk has been modified and needs to be saved
    pub dirty: bool,
    /// The parent entity for this chunk that contains all chunk-related entities (if spawned)
    pub parent_entity: Option<Entity>,
}

impl ChunkData {
    /// Get the tile at the given local coordinates within this chunk
    fn get_tile(&self, local_x: u32, local_y: u32) -> Option<TileType> {
        if local_x < CHUNK_SIZE && local_y < CHUNK_SIZE {
            Some(self.tiles[local_y as usize][local_x as usize])
        } else {
            None
        }
    }
}

/// Component to mark chunk tilemap entities
#[derive(Component)]
struct ChunkTilemap {
    chunk_coord: ChunkCoord,
}

/// Component to mark chunk parent entities that hold all chunk-related entities
#[derive(Component)]
struct ChunkParent {
    pub chunk_coord: ChunkCoord,
}
