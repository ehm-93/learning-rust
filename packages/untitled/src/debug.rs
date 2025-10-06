//! Debug overlay system for F3-style debug information display
//!
//! Provides a comprehensive debug overlay showing:
//! - Player coordinates and chunk information
//! - Frame time, FPS, and performance metrics
//! - Chunk boundaries visualization
//! - Active game state information

use bevy::{
    prelude::*,
    diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore},
};

use crate::{
    player::Player,
    world::chunks::{world_pos_to_chunk_coord, ChunkCoord, CHUNK_SIZE, ChunkingState},
    components::MainCamera,
    resources::GameState,
};

/// Plugin for the debug overlay system
pub struct DebugOverlayPlugin;

impl Plugin for DebugOverlayPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add frame time diagnostics plugin for FPS tracking
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            // Initialize debug state
            .init_resource::<DebugOverlayState>()
            // Add debug overlay systems
            .add_systems(Startup, setup_debug_overlay)
            .add_systems(Update, (
                // Toggle debug overlay with F3 key
                toggle_debug_overlay,
            ))
            .add_systems(FixedUpdate, (
                // Update debug information when overlay is visible
                update_debug_text.run_if(resource_exists::<DebugOverlayVisible>),
                // Render chunk boundaries when overlay is visible
                render_chunk_boundaries.run_if(resource_exists::<DebugOverlayVisible>),
            ));
    }
}

/// Resource to track debug overlay state
#[derive(Resource, Default)]
pub struct DebugOverlayState {
    pub show_overlay: bool,
    pub show_chunk_boundaries: bool,
}

/// Marker resource indicating debug overlay is visible
#[derive(Resource)]
pub struct DebugOverlayVisible;

/// Component for the debug overlay UI container
#[derive(Component)]
pub struct DebugOverlay;

/// Component for the debug text display
#[derive(Component)]
pub struct DebugText;

/// Component for chunk boundary lines
#[derive(Component)]
pub struct ChunkBoundaryLine;

/// Sets up the debug overlay UI elements (initially hidden)
fn setup_debug_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            width: Val::Px(450.0),
            height: Val::Px(500.0), // Increased height to show more debug info
            padding: UiRect::all(Val::Px(10.0)),
            display: Display::None, // Initially hidden
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        BorderRadius::all(Val::Px(5.0)),
        DebugOverlay,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Debug Information"),
            TextFont {
                font_size: 12.0, // Smaller font to fit more text
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                overflow: Overflow::clip(), // Clip overflow content
                ..default()
            },
            DebugText,
        ));
    });
}

/// System to toggle debug overlay visibility with F3 key
fn toggle_debug_overlay(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugOverlayState>,
    mut debug_overlay_query: Query<&mut Node, With<DebugOverlay>>,
    chunk_boundary_query: Query<Entity, With<ChunkBoundaryLine>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_state.show_overlay = !debug_state.show_overlay;

        // Update UI visibility
        if let Ok(mut overlay_style) = debug_overlay_query.single_mut() {
            overlay_style.display = if debug_state.show_overlay {
                Display::Flex
            } else {
                Display::None
            };
        }

        // Manage debug overlay visible resource
        if debug_state.show_overlay {
            commands.insert_resource(DebugOverlayVisible);
        } else {
            commands.remove_resource::<DebugOverlayVisible>();

            // Clean up chunk boundary lines when hiding overlay
            for entity in chunk_boundary_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System to update debug information text
fn update_debug_text(
    mut debug_text_query: Query<&mut Text, With<DebugText>>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, (With<MainCamera>, Without<Player>)>,
    chunk_registry: Res<crate::world::chunks::ChunkRegistry>,
    chunking_state: Res<State<ChunkingState>>,
    game_state: Res<GameState>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
) {
    if let Ok(mut text) = debug_text_query.single_mut() {
        let mut debug_info = String::new();

        // Header
        debug_info.push_str("=== Debug Information (F3) ===\n\n");

        // Frame time and FPS information
        if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diag.smoothed() {
                debug_info.push_str(&format!("FPS: {:.1}\n", fps_smoothed));
            }
        }

        if let Some(frame_time_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
            if let Some(frame_time) = frame_time_diag.smoothed() {
                debug_info.push_str(&format!("Frame Time: {:.2}ms\n", frame_time));
            }
        }

        debug_info.push_str(&format!("Delta Time: {:.2}ms\n", time.delta_secs() * 1000.0));
        debug_info.push('\n');

        // Player position information
        if let Ok(player_transform) = player_query.single() {
            let player_pos = player_transform.translation.truncate();
            debug_info.push_str(&format!("Player Position:\n"));
            debug_info.push_str(&format!("  World: ({:.1}, {:.1})\n", player_pos.x, player_pos.y));

            // Convert to tile coordinates
            let tile_x = (player_pos.x / 16.0).floor() as i32;
            let tile_y = (player_pos.y / 16.0).floor() as i32;
            debug_info.push_str(&format!("  Tile: ({}, {})\n", tile_x, tile_y));

            // Chunk information
            let chunk_coord = world_pos_to_chunk_coord(player_pos);
            debug_info.push_str(&format!("  Chunk: ({}, {})\n", chunk_coord.x, chunk_coord.y));

            // Local position within chunk
            let chunk_size_world = CHUNK_SIZE as f32 * 16.0;
            let chunk_world_pos = Vec2::new(
                chunk_coord.x as f32 * chunk_size_world,
                chunk_coord.y as f32 * chunk_size_world,
            );
            let local_pos = player_pos - chunk_world_pos;
            let local_tile_x = (local_pos.x / 16.0).floor() as i32;
            let local_tile_y = (local_pos.y / 16.0).floor() as i32;
            debug_info.push_str(&format!("  Local Tile: ({}, {})\n", local_tile_x, local_tile_y));
        }

        debug_info.push('\n');

        // Camera information
        if let Ok(camera_transform) = camera_query.single() {
            let cam_pos = camera_transform.translation.truncate();
            let cam_scale = camera_transform.scale.truncate();
            debug_info.push_str(&format!("Camera:\n"));
            debug_info.push_str(&format!("  Position: ({:.1}, {:.1})\n", cam_pos.x, cam_pos.y));
            debug_info.push_str(&format!("  Scale: ({:.2}, {:.2})\n", cam_scale.x, cam_scale.y));
        }

        debug_info.push('\n');

        // Chunk system information
        debug_info.push_str(&format!("Chunks:\n"));
        debug_info.push_str(&format!("  State: {:?}\n", chunking_state.get()));

        // Get active chunks from the new ChunkRegistry
        let mut loaded_chunks: Vec<_> = chunk_registry.active_chunks().collect();
        debug_info.push_str(&format!("  Active: {}\n", loaded_chunks.len()));

        // List loaded chunk coordinates (limit to prevent text overflow)
        loaded_chunks.sort_by_key(|coord| (coord.x, coord.y));

        if loaded_chunks.len() <= 10 && loaded_chunks.len() > 0 {
            debug_info.push_str("  Coordinates: ");
            for (i, coord) in loaded_chunks.iter().enumerate() {
                if i > 0 { debug_info.push_str(", "); }
                debug_info.push_str(&format!("({}, {})", coord.x, coord.y));
            }
            debug_info.push('\n');
        } else if loaded_chunks.len() > 10 {
            debug_info.push_str(&format!("  (Too many chunks to list: {})\n", loaded_chunks.len()));
        } else {
            debug_info.push_str("  (No chunks loaded)\n");
        }

        debug_info.push('\n');

        // Game state information
        debug_info.push_str(&format!("Game State: {:?}\n", *game_state));

        **text = debug_info;
    }
}

/// System to render chunk boundaries as lines when debug overlay is active
fn render_chunk_boundaries(
    mut gizmos: Gizmos,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, (With<MainCamera>, Without<Player>)>,
    chunk_registry: Res<crate::world::chunks::ChunkRegistry>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation.truncate();

        // Get camera position and scale for determining which chunks to draw
        let (cam_pos, cam_scale) = if let Ok(camera_transform) = camera_query.single() {
            (
                camera_transform.translation.truncate(),
                camera_transform.scale.truncate(),
            )
        } else {
            (player_pos, Vec2::ONE)
        };

        // Calculate visible area (with some margin)
        let visible_radius = 10_000.0 / cam_scale.x.min(cam_scale.y); // Adjust based on zoom
        let min_pos = cam_pos - Vec2::splat(visible_radius);
        let max_pos = cam_pos + Vec2::splat(visible_radius);

        // Convert to chunk coordinates
        let min_chunk = world_pos_to_chunk_coord(min_pos);
        let max_chunk = world_pos_to_chunk_coord(max_pos);

        let chunk_size_world = CHUNK_SIZE as f32 * 16.0;

        // Draw grid lines for chunks in visible area
        for chunk_x in (min_chunk.x - 1)..=(max_chunk.x + 1) {
            for chunk_y in (min_chunk.y - 1)..=(max_chunk.y + 1) {
                let chunk_coord = ChunkCoord::new(chunk_x, chunk_y);
                let chunk_world_pos = Vec2::new(
                    chunk_coord.x as f32 * chunk_size_world,
                    chunk_coord.y as f32 * chunk_size_world,
                );

                // Determine line color based on chunk registry state
                let is_active = chunk_registry.get_refcount(chunk_coord) > 0;

                let line_color = if is_active {
                    Color::srgb(0.0, 1.0, 0.0) // Green for active chunks
                } else {
                    Color::srgb(0.5, 0.5, 0.5) // Gray for inactive chunks
                };

                // Draw chunk boundary rectangle
                let top_left = chunk_world_pos;
                let top_right = chunk_world_pos + Vec2::new(chunk_size_world, 0.0);
                let bottom_right = chunk_world_pos + Vec2::new(chunk_size_world, chunk_size_world);
                let bottom_left = chunk_world_pos + Vec2::new(0.0, chunk_size_world);

                // Draw the four sides of the chunk boundary
                gizmos.line_2d(top_left, top_right, line_color);
                gizmos.line_2d(top_right, bottom_right, line_color);
                gizmos.line_2d(bottom_right, bottom_left, line_color);
                gizmos.line_2d(bottom_left, top_left, line_color);

                // Draw chunk coordinate label if chunk is close to player
                let chunk_center = chunk_world_pos + Vec2::splat(chunk_size_world * 0.5);
                let distance_to_player = (chunk_center - player_pos).length();

                if distance_to_player < chunk_size_world * 3.0 {
                    // Draw chunk coordinate at center (this will be a simple cross for now)
                    let cross_size = 32.0;
                    gizmos.line_2d(
                        chunk_center - Vec2::new(cross_size, 0.0),
                        chunk_center + Vec2::new(cross_size, 0.0),
                        Color::srgb(1.0, 1.0, 0.0),
                    );
                    gizmos.line_2d(
                        chunk_center - Vec2::new(0.0, cross_size),
                        chunk_center + Vec2::new(0.0, cross_size),
                        Color::srgb(1.0, 1.0, 0.0),
                    );
                }
            }
        }

        // Draw current player chunk highlight
        let player_chunk = world_pos_to_chunk_coord(player_pos);
        let player_chunk_world_pos = Vec2::new(
            player_chunk.x as f32 * chunk_size_world,
            player_chunk.y as f32 * chunk_size_world,
        );

        // Highlight current chunk with a thicker, brighter border
        let highlight_color = Color::srgb(1.0, 0.5, 0.0); // Orange
        let top_left = player_chunk_world_pos;
        let top_right = player_chunk_world_pos + Vec2::new(chunk_size_world, 0.0);
        let bottom_right = player_chunk_world_pos + Vec2::new(chunk_size_world, chunk_size_world);
        let bottom_left = player_chunk_world_pos + Vec2::new(0.0, chunk_size_world);

        // Draw thicker lines for current chunk
        for offset in [-2.0, -1.0, 0.0, 1.0, 2.0] {
            gizmos.line_2d(
                top_left + Vec2::new(0.0, offset),
                top_right + Vec2::new(0.0, offset),
                highlight_color,
            );
            gizmos.line_2d(
                top_right + Vec2::new(offset, 0.0),
                bottom_right + Vec2::new(offset, 0.0),
                highlight_color,
            );
            gizmos.line_2d(
                bottom_right + Vec2::new(0.0, offset),
                bottom_left + Vec2::new(0.0, offset),
                highlight_color,
            );
            gizmos.line_2d(
                bottom_left + Vec2::new(offset, 0.0),
                top_left + Vec2::new(offset, 0.0),
                highlight_color,
            );
        }
    }
}
