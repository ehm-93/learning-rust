//! Event-driven terrain chunk management
//!
//! This module handles terrain-specific chunk loading/unloading by subscribing
//! to chunk events from the core chunking system.
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_ecs_tilemap::prelude::*;
use std::collections::HashMap;

use crate::world::chunks::*;
use crate::world::tiles::{TileType, TILE_SIZE};
use crate::world::scenes::dungeon::resources::DungeonState;
use crate::world::constants::MACRO_PX_PER_CHUNK;
use crate::world::PX_PER_TILE;
use crate::persistence::ChunkDatabase;

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

/// Minimum number of triangles required to create a valid trimesh collider
const MIN_TRIMESH_TRIANGLES: usize = 1;

// === Noise Scaling Constants ===

/// Scale factor to convert noise from [0,1] to [-1,1] range
const NOISE_SCALE_FACTOR: f32 = 2.0;
/// Offset to center noise around zero
const NOISE_OFFSET: f32 = 1.0;

/// Budget (in seconds) for chunk loading per frame to avoid frame drops
const CHUNK_LOADING_BUDGET: f32 = 0.004;

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

    /// Iterate over all loaded chunks and their tile data
    pub fn iter_loaded(&self) -> impl Iterator<Item = (ChunkCoord, &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize])> {
        self.chunks.iter().filter_map(|(coord, state)| {
            if let TerrainChunkState::Loaded { tiles, .. } = state {
                Some((*coord, tiles))
            } else {
                None
            }
        })
    }
}

/// System that listens for LoadChunk events and starts async terrain generation
pub fn handle_chunk_load_events(
    mut terrain_chunks: ResMut<TerrainChunks>,
    mut load_events: EventReader<LoadChunk>,
    mut preload_events: EventReader<PreloadChunk>,
    dungeon_state: Res<DungeonState>,
    db: Option<Res<ChunkDatabase>>,
) {
    // Process critical load events first (higher priority)
    for event in load_events.read() {
        handle_single_load_event(&mut terrain_chunks, event.pos, &dungeon_state, db.as_deref());
    }

    // Then process preload events (lower priority)
    for event in preload_events.read() {
        handle_single_load_event(&mut terrain_chunks, event.pos, &dungeon_state, db.as_deref());
    }
}

/// Helper function to handle a single chunk load event
fn handle_single_load_event(
    terrain_chunks: &mut TerrainChunks,
    chunk_coord: ChunkCoord,
    dungeon_state: &DungeonState,
    db: Option<&ChunkDatabase>,
) {
    // Check if chunk is already loading or loaded
    if terrain_chunks.chunks.contains_key(&chunk_coord) {
        return; // Already being handled
    }

    // Try to load from database first
    if let Some(database) = db {
        if let Ok(Some(tiles)) = database.load_terrain_chunk(dungeon_state.map_id, chunk_coord) {
            info!("Loaded terrain chunk {:?} from database for map {}", chunk_coord, dungeon_state.map_id);
            // Spawn the terrain entities immediately with loaded data
            // Note: This will be handled by a separate system to avoid blocking
            // For now, we'll still use the async task system but with pre-loaded data
            let task_pool = AsyncComputeTaskPool::get();
            let task = task_pool.spawn(async move {
                ChunkData {
                    position: chunk_coord,
                    tiles,
                }
            });
            terrain_chunks.chunks.insert(chunk_coord, TerrainChunkState::Loading { task });
            return;
        }
    }

    // No saved data found, generate new terrain data
    let task_pool = AsyncComputeTaskPool::get();
    let macro_map = dungeon_state.macro_map.clone();

    let task = task_pool.spawn(async move {
        // Generate tile data (reuse existing logic from ChunkManager)
        let tiles = generate_chunk_tiles(chunk_coord, &macro_map);

        ChunkData {
            position: chunk_coord,
            tiles,
        }
    });

    terrain_chunks.chunks.insert(chunk_coord, TerrainChunkState::Loading { task });
}

/// System that checks async tasks and spawns terrain entities when complete
pub fn poll_terrain_loading_tasks(
    mut commands: Commands,
    mut terrain_chunks: ResMut<TerrainChunks>,
    mut frame_debt: Local<f32>,
    chunk_loaders: Query<&Transform, With<ChunkLoader>>,
) {
    if 0.0 < *frame_debt {
        // Skip processing this frame to catch up
        *frame_debt -= CHUNK_LOADING_BUDGET;
        *frame_debt = frame_debt.max(0.0);
        return;
    }

    let mut completed_chunks = Vec::new();
    let start_time = std::time::Instant::now();

    // Check which tasks have completed
    for (chunk_coord, state) in terrain_chunks.chunks.iter_mut() {
        if let TerrainChunkState::Loading { task } = state {
            if task.is_finished() {
                completed_chunks.push(*chunk_coord);
            }
        }
    }

    // Sort by distance to chunk loaders
    let loaders = chunk_loaders.iter().collect::<Vec<&Transform>>();
    completed_chunks.sort_by_key(|&chunk_coord| {
        loaders.iter().map(|loader| {
            let dx = loader.translation.x - (chunk_coord.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE);
            let dy = loader.translation.y - (chunk_coord.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE);
            dx * dx + dy * dy // Squared distance
        }).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(f32::MAX) as i32 // Use minimum distance to any loader
    });

    // Process completed chunks, closest first, within time budget
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
                }
            }
        }
        // Check elapsed time and enforce budget
        let elapsed_time = start_time.elapsed();
        if elapsed_time.as_secs_f32() > CHUNK_LOADING_BUDGET {
            *frame_debt += elapsed_time.as_secs_f32() - CHUNK_LOADING_BUDGET;
            break;
        }
    }
}

/// System that listens for UnloadChunk events and despawns terrain
pub fn handle_chunk_unload_events(
    mut commands: Commands,
    mut terrain_chunks: ResMut<TerrainChunks>,
    mut unload_events: EventReader<UnloadChunk>,
    dungeon_state: Res<DungeonState>,
    db: Option<Res<ChunkDatabase>>,
) {
    for event in unload_events.read() {
        let chunk_coord = event.pos;

        if let Some(state) = terrain_chunks.chunks.remove(&chunk_coord) {
            match state {
                TerrainChunkState::Loading { task: _ } => {
                    // Task will be dropped automatically, canceling the async work
                }
                TerrainChunkState::Loaded { entity, tiles } => {
                    // Save terrain data to database before unloading
                    if let Some(database) = db.as_deref() {
                        if let Err(e) = database.save_terrain_chunk(dungeon_state.map_id, chunk_coord, &tiles) {
                            error!("Failed to save terrain chunk {:?}: {}", chunk_coord, e);
                        } else {
                            info!("Saved terrain chunk {:?} to database for map {}", chunk_coord, dungeon_state.map_id);
                        }
                    }

                    // Despawn the terrain entity tree
                    commands.entity(entity).despawn();
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
    let half_chunk_size = (CHUNK_SIZE as f32 * TILE_SIZE) * 0.5;

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

    // Generate trimesh collider for all wall tiles in this chunk
    if let Some((vertices, triangles)) = generate_wall_trimesh(&chunk_data.tiles) {
        if triangles.len() >= MIN_TRIMESH_TRIANGLES {
            // Create a trimesh collider from the vertices and triangles
            match Collider::trimesh(vertices, triangles) {
                Ok(collider) => {
                    let wall_collider = commands.spawn((
                        crate::world::tiles::WallTile,
                        collider,
                        RigidBody::Fixed,
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Hidden, // Invisible collider - visual handled by tile
                        ChunkTilemap { chunk_coord: chunk_data.position }, // Tag for cleanup
                    )).id();

                    commands.entity(parent_entity).add_child(wall_collider);
                }
                Err(e) => {
                    warn!("Failed to create trimesh collider for chunk {:?}: {:?}", chunk_data.position, e);
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

    parent_entity
}

/// Represents a rectangular region of contiguous wall tiles
#[derive(Debug, Clone)]
struct WallRectangle {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

/// Generate trimesh collider data for wall tiles in a chunk using consolidated rectangles
/// Returns vertices and triangle indices for creating a trimesh collider
fn generate_wall_trimesh(
    tiles: &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
) -> Option<(Vec<Vec2>, Vec<[u32; 3]>)> {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    // Find all rectangular regions of contiguous wall tiles
    let wall_rectangles = find_wall_rectangles(tiles);

    if wall_rectangles.is_empty() {
        return None;
    }

    // Calculate offset to match tilemap positioning
    let half_chunk_size = (CHUNK_SIZE as f32 * TILE_SIZE) * 0.5;

    // Generate vertices and triangles for each wall rectangle
    for rect in wall_rectangles {
        // Calculate world coordinates for the rectangle corners
        let left = (rect.x as f32 * TILE_SIZE) - half_chunk_size - TILE_SIZE * 0.5;
        let right = ((rect.x + rect.width) as f32 * TILE_SIZE) - half_chunk_size - TILE_SIZE * 0.5;
        let bottom = (rect.y as f32 * TILE_SIZE) - half_chunk_size - TILE_SIZE * 0.5;
        let top = ((rect.y + rect.height) as f32 * TILE_SIZE) - half_chunk_size - TILE_SIZE * 0.5;

        // Add four vertices for this rectangle
        let base_index = vertices.len() as u32;
        vertices.push(Vec2::new(left, bottom));   // bottom-left
        vertices.push(Vec2::new(right, bottom));  // bottom-right
        vertices.push(Vec2::new(left, top));      // top-left
        vertices.push(Vec2::new(right, top));     // top-right

        // Create two triangles for this rectangle
        // Triangle 1: bottom-left, bottom-right, top-left
        triangles.push([base_index, base_index + 1, base_index + 2]);
        // Triangle 2: bottom-right, top-right, top-left
        triangles.push([base_index + 1, base_index + 3, base_index + 2]);
    }

    Some((vertices, triangles))
}

/// Find rectangular regions of contiguous wall tiles using a largest-first rectangle packing algorithm
fn find_wall_rectangles(
    tiles: &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
) -> Vec<WallRectangle> {
    let mut rectangles = Vec::new();
    let mut processed = vec![vec![false; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

    // Repeat until all wall tiles are covered
    while has_unprocessed_wall_tiles(tiles, &processed) {
        // Find the largest possible rectangle among all remaining wall tiles
        let largest_rect = find_largest_rectangle_global(tiles, &processed);

        if let Some(rect) = largest_rect {
            // Mark this rectangle as processed
            mark_rectangle_processed(&mut processed, &rect);
            rectangles.push(rect);
        } else {
            // Safety break - shouldn't happen but prevents infinite loop
            break;
        }
    }

    rectangles
}

/// Check if there are any unprocessed wall tiles remaining
fn has_unprocessed_wall_tiles(
    tiles: &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    processed: &Vec<Vec<bool>>,
) -> bool {
    for y in 0..CHUNK_SIZE as usize {
        for x in 0..CHUNK_SIZE as usize {
            if tiles[x][y] == TileType::Wall && !processed[x][y] {
                return true;
            }
        }
    }
    false
}

/// Find the largest possible rectangle among all unprocessed wall tiles
fn find_largest_rectangle_global(
    tiles: &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    processed: &Vec<Vec<bool>>,
) -> Option<WallRectangle> {
    let mut best_rect: Option<WallRectangle> = None;
    let mut best_area = 0;

    // Check every possible starting position
    for y in 0..CHUNK_SIZE as usize {
        for x in 0..CHUNK_SIZE as usize {
            if tiles[x][y] == TileType::Wall && !processed[x][y] {
                // Try to find the largest rectangle starting at this position
                let rect = find_largest_rectangle_at(tiles, processed, x, y);
                let area = rect.width * rect.height;

                if area > best_area {
                    best_area = area;
                    best_rect = Some(rect);
                }
            }
        }
    }

    best_rect
}

/// Find the largest rectangle of wall tiles starting at the given position
fn find_largest_rectangle_at(
    tiles: &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    processed: &Vec<Vec<bool>>,
    start_x: usize,
    start_y: usize,
) -> WallRectangle {
    let mut best_rect = WallRectangle {
        x: start_x,
        y: start_y,
        width: 1,
        height: 1,
    };
    let mut best_area = 1;

    // Try different widths and find the maximum height for each
    for width in 1..=(CHUNK_SIZE as usize - start_x) {
        // Check if this width is valid (all tiles in the row are walls and unprocessed)
        let mut width_valid = true;
        for x in start_x..(start_x + width) {
            if tiles[x][start_y] != TileType::Wall || processed[x][start_y] {
                width_valid = false;
                break;
            }
        }

        if !width_valid {
            break; // Can't extend width further
        }

        // Find maximum height for this width
        let mut height = 1;
        for y in (start_y + 1)..CHUNK_SIZE as usize {
            let mut row_valid = true;
            for x in start_x..(start_x + width) {
                if tiles[x][y] != TileType::Wall || processed[x][y] {
                    row_valid = false;
                    break;
                }
            }
            if row_valid {
                height += 1;
            } else {
                break;
            }
        }

        let area = width * height;
        if area > best_area {
            best_area = area;
            best_rect = WallRectangle {
                x: start_x,
                y: start_y,
                width,
                height,
            };
        }
    }

    best_rect
}

/// Mark all tiles in a rectangle as processed
fn mark_rectangle_processed(
    processed: &mut Vec<Vec<bool>>,
    rect: &WallRectangle,
) {
    for y in rect.y..(rect.y + rect.height) {
        for x in rect.x..(rect.x + rect.width) {
            processed[x][y] = true;
        }
    }
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
            .add_systems(FixedUpdate, (
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
