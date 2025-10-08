//! Fog of War (FOW) System
//!
//! This module implements a sophisticated fog of war system with smooth interpolation,
//! multi-threaded computation, and wall-based line-of-sight blocking.
//!
//! # Architecture
//!
//! The FOW system operates in several stages:
//!
//! 1. **Loading**: When chunks are loaded, FOW data is restored from the database or
//!    initialized to fully fogged state.
//!
//! 2. **Task Spawning**: When revealers (e.g., player) move, background computation tasks
//!    are spawned using Bevy's `AsyncComputeTaskPool` to calculate updated vision.
//!
//! 3. **Vision Calculation**: Background tasks compute two-tier vision:
//!    - **Force Radius**: Inner area always visible regardless of obstacles
//!    - **LOS Radius**: Outer area using raycasting for wall-based line-of-sight
//!
//! 4. **Task Polling**: Completed tasks are polled and their results applied to chunk
//!    `desired_vision` data. Affected chunks are marked with `NeedsLerp`.
//!
//! 5. **Interpolation**: `current_vision` smoothly lerps toward `desired_vision` with
//!    frame budgeting to prevent performance spikes.
//!
//! 6. **Rendering**: FOW overlay textures are generated from interpolated vision data
//!    and rendered as black transparent sprites.
//!
//! 7. **Unloading**: When chunks unload, FOW data is persisted to the database.
//!
//! # Two-Tier Vision
//!
//! The system uses a two-tier approach for more interesting gameplay:
//! - **Force radius** (~12 tiles): Always visible, represents immediate awareness
//! - **LOS radius** (~64 tiles): Requires clear line of sight, blocked by walls
//!
//! Vision strength fades smoothly between these radii and at edges near walls.
//!
//! # Coordinate Systems
//!
//! **CRITICAL**: Different coordinate systems are used throughout:
//! - **Terrain tiles**: Indexed as `tiles[x][y]` (column-major)
//! - **FOW arrays**: Indexed as `vision[y][x]` (row-major)
//! - **Texture rendering**: Y-axis flipped using `.rev()` to match Bevy's bottom-left origin
//!
//! Always verify coordinate ordering when converting between these systems.
//!
//! # Performance
//!
//! - Background calculation prevents main thread blocking
//! - Frame budgeting (`FOW_LERP_BUDGET`) with debt tracking ensures smooth 60fps
//! - `NeedsLerp` marker ensures only changed chunks are processed
//! - Task deduplication prevents redundant calculations

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::AsyncComputeTaskPool;

use crate::world::{chunks::{LoadChunk, UnloadChunk, CHUNK_SIZE}, PX_PER_TILE};
use crate::persistence::ChunkDatabase;

use super::*;

// ========================================
// Constants
// ========================================

/// Z-layer for fog of war rendering (above terrain and entities)
pub const FOW_Z: f32 = 10.0;

/// Chunk size in tiles
const CHUNK_SIZE_TILES: usize = CHUNK_SIZE as usize;

/// Chunk size in pixels
const CHUNK_SIZE_PX: usize = CHUNK_SIZE as usize * PX_PER_TILE;

/// Blur margin as a percentage of LOS radius for smooth edge fade (0.8 = 80%)
const BLUR_MARGIN_PERCENTAGE: f32 = 0.8;

/// Minimum blur margin in tiles to ensure smooth edges even with small LOS radius
const MIN_BLUR_MARGIN: f32 = 16.0;

/// Distance in tiles over which to blend vision near walls for smooth fade
const WALL_BLEND_DISTANCE: f32 = 3.5;

/// Budget (in seconds) for FOW lerping per frame to avoid frame drops
const FOW_LERP_BUDGET: f32 = 0.002;

/// Threshold to consider lerping complete (difference less than this means we're close enough)
const LERP_COMPLETION_THRESHOLD: f32 = 0.5;

// ========================================
// Load/Unload Systems
// ========================================

/// Load FowChunk components for newly loaded chunks
pub fn load_fow_chunks(
    mut commands: Commands,
    mut chunks_query: Query<&mut FowChunk>,
    mut load_chunk: EventReader<LoadChunk>,
    dungeon_state: Res<crate::world::scenes::dungeon::resources::DungeonState>,
    db: Option<Res<ChunkDatabase>>,
) {
    // index all existing chunks by position
    let chunks = chunks_query.iter_mut()
        .map(|c| (c.position, c))
        .collect::<std::collections::HashMap<_, _>>();

    for event in load_chunk.read() {
        if !chunks.contains_key(&event.pos) {
            // Try to load from database first
            let desired_vision = if let Some(database) = db.as_deref() {
                if let Ok(Some(loaded_vision)) = database.load_fow_chunk(dungeon_state.map_id, event.pos) {
                    info!("Loaded FOW chunk {:?} from database for map {}", event.pos, dungeon_state.map_id);
                    loaded_vision
                } else {
                    // No saved data, create fresh vision grid
                    vec![vec![0u8; CHUNK_SIZE_TILES]; CHUNK_SIZE_TILES]
                }
            } else {
                // No database, create fresh vision grid
                vec![vec![0u8; CHUNK_SIZE_TILES]; CHUNK_SIZE_TILES]
            };

            // Create FowChunk with loaded or fresh vision data
            // Initialize current_vision to match desired_vision
            let current_vision: Vec<Vec<f32>> = desired_vision.iter()
                .map(|row| row.iter().map(|&v| v as f32).collect())
                .collect();

            let fow_chunk = FowChunk {
                position: event.pos,
                current_vision,
                desired_vision,
                lerp_speed: 0.1,
            };

            // spawn entity with FowChunk component and required hierarchy components
            commands.spawn((
                fow_chunk,
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
    }
}

/// Unload FowChunk components for unloaded chunks
pub fn unload_fow_chunks(
    mut commands: Commands,
    chunks_query: Query<(Entity, &FowChunk)>,
    mut unload_chunk: EventReader<UnloadChunk>,
    dungeon_state: Res<crate::world::scenes::dungeon::resources::DungeonState>,
    db: Option<Res<ChunkDatabase>>,
) {
    // index all existing chunks by position
    let chunks = chunks_query.iter()
        .map(|(entity, chunk)| (chunk.position, (entity, chunk)))
        .collect::<std::collections::HashMap<_, _>>();

    for event in unload_chunk.read() {
        if let Some((entity, chunk)) = chunks.get(&event.pos) {
            // Save FOW data to database before unloading (use desired_vision as authoritative)
            if let Some(database) = db.as_deref() {
                if let Err(e) = database.save_fow_chunk(dungeon_state.map_id, event.pos, &chunk.desired_vision) {
                    error!("Failed to save FOW chunk {:?}: {}", event.pos, e);
                } else {
                    info!("Saved FOW chunk {:?} to database for map {}", event.pos, dungeon_state.map_id);
                }
            }

            // despawn entity with FowChunk component
            commands.entity(*entity).despawn();
        }
    }
}

/// Pre-calculate a two-tier vision gradient stamp
///
/// This generates a circular vision pattern with:
/// - Inner force radius: always visible regardless of obstacles
/// - Middle transition zone: smooth gradient between force and LOS
/// - Outer LOS radius: requires line-of-sight checks
/// - Edge blur zone: smooth fade to zero for soft edges
///
/// # Arguments
/// * `force_radius` - Inner radius that's always visible (in tiles)
/// * `force_strength` - Visibility strength in force zone (0.0-1.0)
/// * `los_radius` - Outer radius requiring line-of-sight (in tiles)
/// * `los_strength` - Visibility strength in LOS zone (0.0-1.0)
///
/// # Returns
/// * Visibility stamp: 2D array where 255 = fully revealed, 0 = fully fogged
/// * LOS mask: indicates which tiles need line-of-sight checks
fn create_vision_stamps(
    force_radius: usize,
    force_strength: f32,
    los_radius: usize,
    los_strength: f32,
) -> (Vec<Vec<u8>>, Vec<Vec<bool>>) {
    // Calculate stamp size with blur margin for soft edges
    let blur_margin = calculate_blur_margin(los_radius);
    let diameter = (los_radius + blur_margin) * 2 + 1;
    let center = (los_radius + blur_margin) as f32;

    let mut visibility_stamp = vec![vec![0u8; diameter]; diameter];
    let mut los_mask = vec![vec![false; diameter]; diameter];

    // Pre-calculate radii as f32 for performance
    let force_radius_f = force_radius as f32;
    let los_radius_f = los_radius as f32;
    let blur_margin_f = blur_margin as f32;
    let transition_distance = los_radius_f - force_radius_f;

    for y in 0..diameter {
        for x in 0..diameter {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();

            let (visibility, needs_los_check) = calculate_tile_visibility(
                distance,
                force_radius_f,
                force_strength,
                los_radius_f,
                los_strength,
                blur_margin_f,
                transition_distance,
            );

            visibility_stamp[y][x] = visibility;
            los_mask[y][x] = needs_los_check;
        }
    }

    (visibility_stamp, los_mask)
}

/// Calculate blur margin for smooth edge fade
fn calculate_blur_margin(los_radius: usize) -> usize {
    (los_radius as f32 * BLUR_MARGIN_PERCENTAGE)
        .max(MIN_BLUR_MARGIN) as usize
}

/// Calculate visibility for a single tile based on distance from center
///
/// Returns (visibility_value, needs_los_check)
fn calculate_tile_visibility(
    distance: f32,
    force_radius: f32,
    force_strength: f32,
    los_radius: f32,
    los_strength: f32,
    blur_margin: f32,
    transition_distance: f32,
) -> (u8, bool) {
    if distance <= force_radius {
        // Within force radius - full strength, always visible
        let visibility = (255.0 * force_strength) as u8;
        (visibility, false)
    } else if distance <= los_radius {
        // Transition zone - smooth gradient from force_strength to los_strength
        let position_in_transition = distance - force_radius;
        let fade = 1.0 - (position_in_transition / transition_distance);
        let strength = force_strength * fade + los_strength * (1.0 - fade);
        let visibility = (255.0 * strength) as u8;
        (visibility, true) // Needs line-of-sight check
    } else if distance <= los_radius + blur_margin {
        // Blur zone - smooth fade to zero for soft edges
        let fade = (los_radius + blur_margin - distance) / blur_margin;
        let visibility = (fade.clamp(0.0, 1.0) * 255.0 * los_strength) as u8;
        (visibility, false)
    } else {
        // Beyond all zones - completely fogged
        (0, false)
    }
}

/// Spawn FOW calculation tasks for revealers that have moved
///
/// When a revealer (e.g., player) moves, this system:
/// 1. Creates a terrain snapshot for raycasting
/// 2. Spawns async tasks on the compute pool
/// 3. Prevents duplicate tasks for the same revealer
pub fn spawn_fow_calculation_tasks(
    mut commands: Commands,
    revealers_query: Query<(Entity, &Transform, &FowRevealer), Changed<Transform>>,
    mut task_set: ResMut<FowTaskSet>,
    terrain_chunks: Res<crate::world::scenes::dungeon::terrain::TerrainChunks>,
) {
    use std::collections::HashMap;
    use std::sync::Arc;

    let revealer_count = revealers_query.iter().count();
    if revealer_count == 0 {
        return;
    }

    // Create a snapshot of terrain data (walls) for raycasting
    // NOTE: terrain tiles use [x][y] indexing, but we store as [y][x] for FOW consistency
    let mut terrain_snapshot = HashMap::new();

    for (chunk_pos, tiles) in terrain_chunks.iter_loaded() {
        let mut wall_data = vec![vec![false; CHUNK_SIZE_TILES]; CHUNK_SIZE_TILES];
        for y in 0..CHUNK_SIZE_TILES {
            for x in 0..CHUNK_SIZE_TILES {
                // Convert TileType::Wall to true (blocks LOS), TileType::Floor to false
                wall_data[y][x] = tiles[x][y] == crate::world::tiles::TileType::Wall;
            }
        }
        terrain_snapshot.insert(chunk_pos, wall_data);
    }

    let terrain_snapshot = Arc::new(terrain_snapshot);
    let task_pool = AsyncComputeTaskPool::get();
    let mut spawned_count = 0;

    for (entity, transform, revealer) in revealers_query.iter() {
        // Skip if this revealer already has an active task
        if task_set.active_tasks.contains(&entity) {
            trace!("Skipping FOW task for entity {:?} - already active", entity);
            continue;
        }

        let work_item = FowWorkItem {
            revealer_pos: transform.translation.truncate(),
            force_radius: revealer.radius,
            force_strength: revealer.strength,
            los_radius: revealer.los_radius,
            los_strength: revealer.los_strength,
            terrain_snapshot: terrain_snapshot.clone(),
        };

        // Spawn the task on async compute pool
        let task = task_pool.spawn(async move {
            calculate_fow_for_work_item(&work_item)
        });

        // Track this task
        task_set.active_tasks.insert(entity);
        spawned_count += 1;

        // Attach task component to the revealer entity
        commands.entity(entity).insert(FowCalculationTask {
            task,
            revealer_entity: entity,
        });

        trace!(
            "Spawned FOW calculation task for entity {:?} at pos {:?}",
            entity,
            transform.translation.truncate()
        );
    }

    if spawned_count > 0 {
        debug!("Spawned {} FOW calculation tasks", spawned_count);
    }
}

// ========================================
// Vision Calculation
// ========================================

/// Process FOW calculations in background (call this from async compute or separate thread)
fn calculate_fow_for_work_item(work_item: &FowWorkItem) -> FowWorkResult {
    use std::collections::HashMap;

    let revealer_tile_x = (work_item.revealer_pos.x / PX_PER_TILE as f32).round() as i32;
    let revealer_tile_y = (work_item.revealer_pos.y / PX_PER_TILE as f32).round() as i32;

    // Calculate the actual stamp size including blur margin
    let blur_margin = (work_item.los_radius as f32 * BLUR_MARGIN_PERCENTAGE).max(MIN_BLUR_MARGIN) as i32;
    let stamp_radius = (work_item.los_radius as i32) + blur_margin;

    // Pre-calculate the vision gradient stamps
    let (vision_stamp, _needs_los_mask) = create_vision_stamps(
        work_item.force_radius,
        work_item.force_strength,
        work_item.los_radius,
        work_item.los_strength,
    );

    // Calculate the range of chunks that might be affected
    let min_chunk_x = ((revealer_tile_x - stamp_radius) as f32 / CHUNK_SIZE_TILES as f32).floor() as i32;
    let max_chunk_x = ((revealer_tile_x + stamp_radius) as f32 / CHUNK_SIZE_TILES as f32).floor() as i32;
    let min_chunk_y = ((revealer_tile_y - stamp_radius) as f32 / CHUNK_SIZE_TILES as f32).floor() as i32;
    let max_chunk_y = ((revealer_tile_y + stamp_radius) as f32 / CHUNK_SIZE_TILES as f32).floor() as i32;

    let mut chunk_updates = HashMap::new();

    // Iterate through all potentially affected chunks
    for chunk_y in min_chunk_y..=max_chunk_y {
        for chunk_x in min_chunk_x..=max_chunk_x {
            let chunk_pos = IVec2::new(chunk_x, chunk_y);
            let chunk_offset_x = chunk_x * CHUNK_SIZE_TILES as i32;
            let chunk_offset_y = chunk_y * CHUNK_SIZE_TILES as i32;

            // Create a vision update for this chunk
            let mut chunk_vision = vec![vec![0u8; CHUNK_SIZE_TILES]; CHUNK_SIZE_TILES];

            // Calculate the range of tiles within this chunk to check
            let start_x = (revealer_tile_x - stamp_radius - chunk_offset_x).max(0);
            let end_x = (revealer_tile_x + stamp_radius - chunk_offset_x).min(CHUNK_SIZE_TILES as i32 - 1);
            let start_y = (revealer_tile_y - stamp_radius - chunk_offset_y).max(0);
            let end_y = (revealer_tile_y + stamp_radius - chunk_offset_y).min(CHUNK_SIZE_TILES as i32 - 1);

            // Apply the vision stamp to the chunk with raycasting for LOS
            for local_y in start_y..=end_y {
                for local_x in start_x..=end_x {
                    let world_tile_x = chunk_offset_x + local_x;
                    let world_tile_y = chunk_offset_y + local_y;

                    // Calculate position in the vision stamp
                    let stamp_x = (world_tile_x - revealer_tile_x + stamp_radius) as usize;
                    let stamp_y = (world_tile_y - revealer_tile_y + stamp_radius) as usize;

                    // Get base stamp visibility with bounds checking
                    let stamp_visibility = vision_stamp.get(stamp_y)
                        .and_then(|row| row.get(stamp_x))
                        .copied()
                        .unwrap_or(0); // If out of bounds, treat as fogged

                    // Calculate distance from revealer
                    let dx = world_tile_x - revealer_tile_x;
                    let dy = world_tile_y - revealer_tile_y;
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    let force_radius_sq = (work_item.force_radius as f32).powi(2);
                    let distance_sq = distance * distance;

                    // Check if we need to raycast for line of sight
                    let visibility = if distance_sq <= force_radius_sq {
                        // Within force radius - always visible
                        stamp_visibility
                    } else if stamp_visibility > 0 {
                        // Beyond force radius - check line of sight with distance-based blending
                        let ray_distance = super::raycasting::cast_ray_distance(
                            &work_item.terrain_snapshot,
                            revealer_tile_x,
                            revealer_tile_y,
                            world_tile_x,
                            world_tile_y,
                            CHUNK_SIZE_TILES
                        );

                        if ray_distance >= distance {
                            // Clear line of sight - use full stamp visibility
                            stamp_visibility
                        } else {
                            // Hit a wall before reaching target - blend based on how close we got
                            let distance_from_wall = distance - ray_distance;

                            if distance_from_wall < WALL_BLEND_DISTANCE {
                                // Apply smooth fade out near wall
                                let fade = 1.0 - (distance_from_wall / WALL_BLEND_DISTANCE).clamp(0.0, 1.0);
                                let blended_visibility = (stamp_visibility as f32 * fade) as u8;
                                blended_visibility
                            } else {
                                0 // Too far past the wall, completely blocked
                            }
                        }
                    } else {
                        0
                    };

                    chunk_vision[local_y as usize][local_x as usize] = visibility;
                }
            }

            chunk_updates.insert(chunk_pos, chunk_vision);
        }
    }

    FowWorkResult { chunk_updates }
}

/// Poll active FOW calculation tasks and apply results when complete
///
/// Checks for finished tasks, applies their vision updates to chunks,
/// and marks affected chunks with `NeedsLerp` for interpolation.
pub fn poll_fow_calculation_tasks(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut FowCalculationTask)>,
    mut chunks_query: Query<(Entity, &mut FowChunk)>,
    mut task_set: ResMut<FowTaskSet>,
) {
    let mut completed_tasks = Vec::new();

    // Check which tasks have completed
    for (entity, task) in task_query.iter() {
        if task.task.is_finished() {
            completed_tasks.push(entity);
        }
    }

    if completed_tasks.is_empty() {
        return;
    }

    debug!("Polling {} completed FOW calculation tasks", completed_tasks.len());

    // Index chunks by position for fast lookup
    let mut chunks: std::collections::HashMap<_, _> = chunks_query
        .iter_mut()
        .map(|(entity, chunk)| (chunk.position, (entity, chunk)))
        .collect();

    // Track chunks that need lerp markers
    let mut chunks_needing_lerp = Vec::new();
    let mut total_chunks_updated = 0;

    // Process completed tasks
    for entity in completed_tasks {
        if let Ok((task_entity, mut task)) = task_query.get_mut(entity) {
            // Block on the task to get the result
            let result = bevy::tasks::block_on(&mut task.task);

            trace!(
                "Task for entity {:?} completed, updating {} chunks",
                task_entity,
                result.chunk_updates.len()
            );

            // Apply the result to chunks
            for (chunk_pos, chunk_vision) in result.chunk_updates {
                if let Some((chunk_entity, chunk)) = chunks.get_mut(&chunk_pos) {
                    // Update the desired_vision by taking the maximum of current and new values
                    // This ensures vision never "forgets" explored areas
                    for y in 0..CHUNK_SIZE_TILES {
                        for x in 0..CHUNK_SIZE_TILES {
                            chunk.desired_vision[y][x] =
                                chunk.desired_vision[y][x].max(chunk_vision[y][x]);
                        }
                    }
                    // Track this chunk for lerp marking
                    chunks_needing_lerp.push(*chunk_entity);
                    total_chunks_updated += 1;
                } else {
                    trace!(
                        "Chunk at {:?} from FOW task result not found (may have been unloaded)",
                        chunk_pos
                    );
                }
            }

            // Remove task component and mark as no longer active
            commands.entity(task_entity).remove::<FowCalculationTask>();
            task_set.active_tasks.remove(&task_entity);
        }
    }

    // Mark chunks that need lerping
    for chunk_entity in chunks_needing_lerp {
        commands.entity(chunk_entity).insert(NeedsLerp);
    }

    debug!(
        "Applied FOW updates to {} chunks, marked for lerping",
        total_chunks_updated
    );
}

/// Lerp current_vision toward desired_vision with strict frame budget and frame debt tracking
///
/// This system smoothly interpolates vision data with performance guarantees:
/// - Only processes chunks marked with `NeedsLerp`
/// - Enforces strict frame budget to prevent frame drops
/// - Tracks frame debt to catch up when behind
/// - Removes `NeedsLerp` marker when interpolation completes
///
/// The lerping happens on the main thread in `FixedUpdate` for deterministic timing.
pub fn lerp_fow_vision(
    mut commands: Commands,
    mut chunks_query: Query<(Entity, &mut FowChunk), With<NeedsLerp>>,
    time: Res<Time>,
    mut frame_debt: Local<f32>,
) {
    // Skip processing if we're behind schedule (frame debt > 0)
    if 0.0 < *frame_debt {
        *frame_debt -= FOW_LERP_BUDGET;
        *frame_debt = frame_debt.max(0.0);
        trace!("Skipping FOW lerp to pay down frame debt: {:.4}s", *frame_debt);
        return;
    }

    let chunk_count = chunks_query.iter().count();
    if chunk_count == 0 {
        return;
    }

    let start_time = std::time::Instant::now();
    let delta_secs = time.delta_secs();
    let mut processed_count = 0;
    let mut completed_count = 0;

    // Process chunks that need lerping within time budget
    for (entity, mut chunk) in chunks_query.iter_mut() {
        // Calculate lerp factor based on lerp_speed and delta time
        // Factor of 10.0 provides responsive but smooth transition
        let lerp_factor = (chunk.lerp_speed * delta_secs * 10.0).min(1.0);

        let mut is_complete = true;

        // Lerp each tile in the chunk
        for y in 0..CHUNK_SIZE_TILES {
            for x in 0..CHUNK_SIZE_TILES {
                let current = chunk.current_vision[y][x];
                let desired = chunk.desired_vision[y][x] as f32;
                let diff = desired - current;

                // Check if this tile still needs lerping
                if diff.abs() > LERP_COMPLETION_THRESHOLD {
                    is_complete = false;
                    // Lerp toward desired value
                    chunk.current_vision[y][x] = current + diff * lerp_factor;
                } else {
                    // Close enough, snap to target
                    chunk.current_vision[y][x] = desired;
                }
            }
        }

        // If chunk is fully lerped, remove the NeedsLerp marker
        if is_complete {
            commands.entity(entity).remove::<NeedsLerp>();
            completed_count += 1;
        }

        processed_count += 1;

        // Enforce frame budget
        let elapsed_time = start_time.elapsed();
        if elapsed_time.as_secs_f32() > FOW_LERP_BUDGET {
            *frame_debt += elapsed_time.as_secs_f32() - FOW_LERP_BUDGET;
            debug!(
                "FOW lerp exceeded budget: processed {}/{} chunks, debt: {:.4}s",
                processed_count, chunk_count, *frame_debt
            );
            break;
        }
    }

    if processed_count > 0 {
        trace!(
            "FOW lerp: processed {} chunks, completed {} ({:.4}s)",
            processed_count,
            completed_count,
            start_time.elapsed().as_secs_f32()
        );
    }
}

/// Draw FOW overlay sprites for chunks
///
/// Creates or updates sprite overlays based on interpolated vision data.
/// Runs in `Update` for smooth visual updates independent of fixed timestep.
pub fn draw_fow(
    mut commands: Commands,
    mut chunks_query: Query<(Entity, &FowChunk), Changed<FowChunk>>,
    mut overlay_query: Query<(&mut Sprite, &FowOverlay)>,
    mut images: ResMut<Assets<Image>>,
) {
    // Index existing overlay entities by chunk position
    let mut overlays = overlay_query
        .iter_mut()
        .map(|(sprite, overlay)| (overlay.position, (sprite, overlay)))
        .collect::<std::collections::HashMap<_, _>>();

    for (chunk_entity, chunk) in chunks_query.iter_mut() {
        if let Some((sprite, _)) = overlays.get_mut(&chunk.position) {
            // Update the sprite's image based on interpolated current_vision
            let vision_texture = create_fow_texture(&chunk.current_vision, &mut images);
            sprite.image = vision_texture;
        } else {
            // Create new overlay sprite
            let vision_texture = create_fow_texture(&chunk.current_vision, &mut images);

            commands.spawn((
                Sprite {
                    image: vision_texture,
                    custom_size: Some(Vec2::new(CHUNK_SIZE_PX as f32, CHUNK_SIZE_PX as f32)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(
                    chunk.position.x as f32 * CHUNK_SIZE_PX as f32
                        + (CHUNK_SIZE_PX as f32 / 2.0),
                    chunk.position.y as f32 * CHUNK_SIZE_PX as f32
                        + (CHUNK_SIZE_PX as f32 / 2.0),
                    FOW_Z,
                )),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                FowOverlay::new(chunk.position),
                ChildOf(chunk_entity),
            ));
        }
    }
}

/// Create a fog of war texture from vision data
///
/// # Coordinate System Notes
///
/// **CRITICAL**: This function handles coordinate system conversions:
/// - Input `vision` array uses row-major indexing: `vision[y][x]`
/// - Bevy's texture coordinate system has origin at bottom-left
/// - We iterate Y in reverse (`.rev()`) to flip the texture vertically
/// - Output pixels are RGBA: black (0,0,0) with inverted alpha
///   - 255 visibility → 0 alpha (transparent/revealed)
///   - 0 visibility → 255 alpha (opaque black/fogged)

fn create_fow_texture(vision: &Vec<Vec<f32>>, assets: &mut Assets<Image>) -> Handle<Image> {
    let height = vision.len();
    let width = if height > 0 { vision[0].len() } else { 0 };

    if width == 0 || height == 0 {
        warn!("Attempted to create FOW texture with invalid dimensions: {}x{}", width, height);
        return Handle::default();
    }

    // Create RGBA image data from interpolated vision data
    // Input: vision[y][x] = 255.0 (fully revealed) to 0.0 (fogged)
    // Output: RGBA where alpha = 0 (transparent) to 255 (opaque black)
    let mut data = Vec::with_capacity(width * height * 4);

    // Flip Y-axis by iterating in reverse to match Bevy's bottom-left origin
    for y in (0..height).rev() {
        for x in 0..width {
            let visibility = vision[y][x].clamp(0.0, 255.0);

            // Invert visibility to alpha: 255 visibility → 0 alpha (revealed)
            let alpha = (255.0 - visibility) as u8;

            // Pure black RGB with inverted alpha creates fog overlay
            data.push(0); // R
            data.push(0); // G
            data.push(0); // B
            data.push(alpha); // A
        }
    }

    // Create the image with Bevy's rendering system
    let image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    );

    assets.add(image)
}
