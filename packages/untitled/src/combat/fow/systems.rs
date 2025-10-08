use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::world::{chunks::{LoadChunk, UnloadChunk, CHUNK_SIZE}, PX_PER_TILE};
use crate::persistence::ChunkDatabase;

use super::*;

pub const FOW_Z: f32 = 10.0;

const CHUNK_SIZE_TILES: usize = CHUNK_SIZE as usize;
const CHUNK_SIZE_PX: usize = CHUNK_SIZE as usize * PX_PER_TILE;

/// Load FowChunk components for newly loaded chunks
pub fn load_fow_chunks(
    mut commands: Commands,
    mut chunks_query: Query<&mut FowChunk>,
    mut load_chunk: EventReader<LoadChunk>,
    db: Option<Res<ChunkDatabase>>,
) {
    // index all existing chunks by position
    let chunks = chunks_query.iter_mut()
        .map(|c| (c.position, c))
        .collect::<std::collections::HashMap<_, _>>();

    for event in load_chunk.read() {
        if !chunks.contains_key(&event.pos) {
            // Try to load from database first
            let vision = if let Some(database) = db.as_deref() {
                if let Ok(Some(loaded_vision)) = database.load_fow_chunk(event.pos) {
                    info!("Loaded FOW chunk {:?} from database", event.pos);
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
            let fow_chunk = FowChunk {
                position: event.pos,
                vision,
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
    db: Option<Res<ChunkDatabase>>,
) {
    // index all existing chunks by position
    let chunks = chunks_query.iter()
        .map(|(entity, chunk)| (chunk.position, (entity, chunk)))
        .collect::<std::collections::HashMap<_, _>>();

    for event in unload_chunk.read() {
        if let Some((entity, chunk)) = chunks.get(&event.pos) {
            // Save FOW data to database before unloading
            if let Some(database) = db.as_deref() {
                if let Err(e) = database.save_fow_chunk(event.pos, &chunk.vision) {
                    error!("Failed to save FOW chunk {:?}: {}", event.pos, e);
                } else {
                    info!("Saved FOW chunk {:?} to database", event.pos);
                }
            }

            // despawn entity with FowChunk component
            commands.entity(*entity).despawn();
        }
    }
}

/// Pre-calculate a vision gradient stamp for a given radius
/// Returns a 2D array where 255 = fully revealed, 0 = fully fogged
fn create_vision_stamp(radius: usize) -> Vec<Vec<u8>> {
    let diameter = radius * 2 + 1;
    let center = radius as f32;
    let mut stamp = vec![vec![0u8; diameter]; diameter];

    let blur_distance = radius as f32 / 2.0; // tiles over which to fade from visible to fogged

    for y in 0..diameter {
        for x in 0..diameter {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();

            // Calculate visibility: 255 at center, fading to 0 at edges
            let visibility = if distance <= radius as f32 - blur_distance {
                255 // Fully visible
            } else if distance >= radius as f32 {
                0 // Fully fogged
            } else {
                // Smooth gradient in between
                let fade = (radius as f32 - distance) / blur_distance;
                (fade.clamp(0.0, 1.0) * 255.0) as u8
            };

            stamp[y][x] = visibility;
        }
    }

    stamp
}

/// Update FowChunk vision based on FowRevealer positions
/// Only runs when revealers move (Changed<Transform> filter)
pub fn update_fow_chunks(
    mut chunks_query: Query<&mut FowChunk>,
    revealers_query: Query<(&Transform, &FowRevealer), Changed<Transform>>,
) {
    // index all existing chunks by position
    let mut chunks = chunks_query.iter_mut()
        .map(|c| (c.position, c))
        .collect::<std::collections::HashMap<_, _>>();

    for (transform, revealer) in revealers_query.iter() {
        // Convert revealer position to tile coordinates
        let revealer_tile_x = (transform.translation.x / PX_PER_TILE as f32).round() as i32;
        let revealer_tile_y = (transform.translation.y / PX_PER_TILE as f32).round() as i32;
        let radius_tiles = revealer.radius as i32;

        // Pre-calculate the vision gradient stamp
        let vision_stamp = create_vision_stamp(revealer.radius);
        let stamp_radius = revealer.radius as i32;

        // Calculate the range of chunks that might be affected
        let min_chunk_x = ((revealer_tile_x - radius_tiles) as f32 / CHUNK_SIZE_TILES as f32).floor() as i32;
        let max_chunk_x = ((revealer_tile_x + radius_tiles) as f32 / CHUNK_SIZE_TILES as f32).floor() as i32;
        let min_chunk_y = ((revealer_tile_y - radius_tiles) as f32 / CHUNK_SIZE_TILES as f32).floor() as i32;
        let max_chunk_y = ((revealer_tile_y + radius_tiles) as f32 / CHUNK_SIZE_TILES as f32).floor() as i32;

        // Iterate through all potentially affected chunks
        for chunk_y in min_chunk_y..=max_chunk_y {
            for chunk_x in min_chunk_x..=max_chunk_x {
                let chunk_pos = IVec2::new(chunk_x, chunk_y);

                if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                    // Calculate the tile coordinates relative to this chunk
                    let chunk_offset_x = chunk_x * CHUNK_SIZE_TILES as i32;
                    let chunk_offset_y = chunk_y * CHUNK_SIZE_TILES as i32;

                    // Calculate the range of tiles within this chunk to check
                    let start_x = (revealer_tile_x - stamp_radius - chunk_offset_x).max(0);
                    let end_x = (revealer_tile_x + stamp_radius - chunk_offset_x).min(CHUNK_SIZE_TILES as i32 - 1);
                    let start_y = (revealer_tile_y - stamp_radius - chunk_offset_y).max(0);
                    let end_y = (revealer_tile_y + stamp_radius - chunk_offset_y).min(CHUNK_SIZE_TILES as i32 - 1);

                    // Apply the vision stamp to the chunk
                    for local_y in start_y..=end_y {
                        for local_x in start_x..=end_x {
                            let world_tile_x = chunk_offset_x + local_x;
                            let world_tile_y = chunk_offset_y + local_y;

                            // Calculate position in the vision stamp
                            let stamp_x = (world_tile_x - revealer_tile_x + stamp_radius) as usize;
                            let stamp_y = (world_tile_y - revealer_tile_y + stamp_radius) as usize;

                            // Get stamp visibility
                            let stamp_visibility = vision_stamp[stamp_y][stamp_x];

                            // Apply stamp: keep maximum visibility (once explored, stays explored)
                            let current_visibility = chunk.vision[local_y as usize][local_x as usize];
                            chunk.vision[local_y as usize][local_x as usize] = current_visibility.max(stamp_visibility);
                        }
                    }
                }
            }
        }
    }
}

/// Draw FowChunk components to the screen
pub fn draw_fow(
    mut commands: Commands,
    mut chunks_query: Query<(Entity, &FowChunk), Changed<FowChunk>>,
    mut overlay_query: Query<(&mut Sprite, &FowOverlay)>,
    mut images: ResMut<Assets<Image>>,
) {
    // index all existing overlay entities by chunk position
    let mut overlays = overlay_query.iter_mut()
        .map(|(sprite, overlay)| (overlay.position, (sprite, overlay)))
        .collect::<std::collections::HashMap<_, _>>();

    for (chunk_entity, chunk) in chunks_query.iter_mut() {
        if let Some((sprite, _)) = overlays.get_mut(&chunk.position) {
            // update the sprite's image based on the chunk's vision
            let vision_texture = create_fow_texture(&chunk.vision, &mut images);
            sprite.image = vision_texture;
        } else {
            // create new overlay sprite
            let vision_texture = create_fow_texture(&chunk.vision, &mut images);

            commands.spawn((
                Sprite {
                    image: vision_texture,
                    custom_size: Some(Vec2::new(
                        CHUNK_SIZE_PX as f32,
                        CHUNK_SIZE_PX as f32,
                    )),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(
                    chunk.position.x as f32 * CHUNK_SIZE_PX as f32 + (CHUNK_SIZE_PX as f32 / 2.0),
                    chunk.position.y as f32 * CHUNK_SIZE_PX as f32 + (CHUNK_SIZE_PX as f32 / 2.0),
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

fn create_fow_texture(vision: &Vec<Vec<u8>>, assets: &mut Assets<Image>) -> Handle<Image> {
    let height = vision.len();
    let width = if height > 0 { vision[0].len() } else { 0 };

    if width == 0 || height == 0 {
        return Handle::default();
    }

    // Create RGBA image data directly from the vision data
    // vision[y][x] contains: 255 = fully revealed (transparent), 0 = fogged (opaque black)
    let mut data = Vec::with_capacity(width * height * 4);

    // Flip Y-axis: iterate from bottom to top to match Bevy's coordinate system
    for y in (0..height).rev() {
        for x in 0..width {
            let visibility = vision[y][x];

            // Invert: 255 visibility -> 0 alpha (transparent), 0 visibility -> 255 alpha (black)
            let alpha = 255 - visibility;

            // Pure black with calculated alpha
            data.push(0);  // R
            data.push(0);  // G
            data.push(0);  // B
            data.push(alpha);  // A
        }
    }

    // Create the image
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
