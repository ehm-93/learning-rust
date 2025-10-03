use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::world::{chunks::ChunkingState, InteractionEvent};

use super::{
    components::*,
    resources::*,
};

/// System to setup the cathedral scene when entering WorldState::Cathedral
pub fn setup_cathedral_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut modifier_system: ResMut<ModifierSystem>,
    mut cathedral_state: ResMut<CathedralState>,
    progression_state: Res<ProgressionState>,
    asset_server: Res<AssetServer>,
    camera_zoom: Res<crate::player::resources::CameraZoom>,
) {
    info!("Entering Cathedral state - setting up scene");

     // Disable chunking in cathedral, fixed map
    commands.set_state(ChunkingState::Disabled);

    // Mark Cathedral as active
    cathedral_state.is_active = true;
    cathedral_state.current_depth = 1; // Default starting depth

    // Spawn camera with persistent zoom level
    let zoom_level = camera_zoom.level;
    commands.spawn((
        Camera2d,
        Transform {
            scale: Vec3::splat(zoom_level),
            ..default()
        },
        crate::components::MainCamera,
        Cathedral, // Tag for cleanup
    ));

    // Spawn player in the center of the tilemap area (now in positive quadrant)
    let tilemap_center_x = (crate::world::tiles::TILEMAP_WIDTH as f32 * crate::world::tiles::TILE_SIZE) / 2.0;
    let tilemap_center_y = (crate::world::tiles::TILEMAP_HEIGHT as f32 * crate::world::tiles::TILE_SIZE) / 2.0;
    let player_entity = commands.spawn(
        crate::player::components::PlayerBundle::new(
            &mut meshes,
            &mut materials,
            Vec3::new(tilemap_center_x, tilemap_center_y, 1.0) // Center of tilemap, Z=1 to render above tilemap at Z=-1
        ),
    ).id();

    // Add cathedral tag for cleanup
    commands.entity(player_entity).insert(Cathedral);

    // Create tilemap directly in cathedral scene
    let texture_handle: Handle<Image> = asset_server.load("sprites/tiles.png");

    let map_size = TilemapSize {
        x: crate::world::tiles::TILEMAP_WIDTH as u32,
        y: crate::world::tiles::TILEMAP_HEIGHT as u32
    };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Generate the test tilemap pattern
    let tilemap_data = crate::world::tiles::generate_test_tilemap();

    // Store wall positions for collision spawning
    let mut wall_positions = Vec::new();

    // Spawn each tile
    for x in 0..crate::world::tiles::TILEMAP_WIDTH {
        for y in 0..crate::world::tiles::TILEMAP_HEIGHT {
            let tile_pos = TilePos { x: x as u32, y: y as u32 };
            let tile_type = tilemap_data[y as usize][x as usize];

            // Set texture index based on tile type
            let texture_index = match tile_type {
                crate::world::tiles::TileType::Floor => TileTextureIndex(0),
                crate::world::tiles::TileType::Wall => TileTextureIndex(1),
            };

            let tile_entity = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index,
                ..Default::default()
            }).id();

            // Mark wall tiles and collect positions for collision
            if tile_type == crate::world::tiles::TileType::Wall {
                commands.entity(tile_entity).insert(crate::world::tiles::WallTile);
                wall_positions.push((x, y));
            }

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Now spawn separate collider entities for each wall
    for (x, y) in wall_positions {
        let world_x = x as f32 * crate::world::tiles::TILE_SIZE;
        let world_y = y as f32 * crate::world::tiles::TILE_SIZE;

        commands.spawn((
            crate::world::tiles::WallTile,
            Collider::cuboid(
                crate::world::tiles::TILE_SIZE / 2.0,
                crate::world::tiles::TILE_SIZE / 2.0
            ),
            RigidBody::Fixed,
            Transform::from_xyz(world_x, world_y, -1.0),
            Visibility::Hidden, // Invisible collider - visual handled by tile
            Cathedral, // Tag for cleanup
        ));
    }

    // Define tilemap properties - everything uses 16x16 consistently
    let grid_size = TilemapGridSize {
        x: crate::world::tiles::TILE_SIZE, // 16.0
        y: crate::world::tiles::TILE_SIZE
    };
    let map_type = TilemapType::Square;
    let tile_size = TilemapTileSize {
        x: crate::world::tiles::TILE_SIZE, // 16.0 - matches sprite atlas
        y: crate::world::tiles::TILE_SIZE
    };

    // Configure the main tilemap entity with consistent sizing
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: Transform::from_xyz(0.0, 0.0, -1.0), // No scaling needed
        ..Default::default()
    })
    .insert(crate::world::tiles::GameTilemap)
    .insert(Cathedral); // Tag for cleanup

    info!("Successfully spawned Cathedral tilemap with {} tiles",
          crate::world::tiles::TILEMAP_WIDTH * crate::world::tiles::TILEMAP_HEIGHT);

    setup_cathedral_entities(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut modifier_system,
        &progression_state,
    );
}

/// System to cleanup the cathedral scene when exiting WorldState::Cathedral
pub fn teardown_cathedral_scene(
    mut commands: Commands,
    mut cathedral_state: ResMut<CathedralState>,
    cathedral_entities: Query<Entity, With<Cathedral>>,
) {
    info!("Leaving Cathedral state - cleaning up scene");

    // Mark Cathedral as inactive
    cathedral_state.is_active = false;

    for entity in cathedral_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// Helper function to set up cathedral entities
fn setup_cathedral_entities(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    _modifier_system: &mut ResMut<ModifierSystem>,
    _progression_state: &Res<ProgressionState>,
) {
    use std::sync::Arc;
    use crate::world::{Interactable, InteractableHighlight, InteractionCallback};

    // Spawn Cathedral marker entity at tilemap center
    let tilemap_width = crate::world::tiles::TILEMAP_WIDTH as f32 * crate::world::tiles::TILE_SIZE;
    let tilemap_height = crate::world::tiles::TILEMAP_HEIGHT as f32 * crate::world::tiles::TILE_SIZE;
    commands.spawn((
        Cathedral,
        Transform::from_translation(Vec3::new(tilemap_width / 2.0, tilemap_height / 2.0, -1.0)),
        Visibility::Visible,
    ));

    // Create three dungeon portals positioned in the center of the 128m×128m box
    let box_center_x = (crate::world::tiles::TILEMAP_WIDTH as f32 * crate::world::tiles::TILE_SIZE) / 2.0;
    let box_center_y = (crate::world::tiles::TILEMAP_HEIGHT as f32 * crate::world::tiles::TILE_SIZE) / 2.0;
    let portal_positions = [
        Vec3::new(box_center_x - 200.0, box_center_y, 0.0), // Left portal
        Vec3::new(box_center_x, box_center_y, 0.0),         // Center portal
        Vec3::new(box_center_x + 200.0, box_center_y, 0.0), // Right portal
    ];

    let portal_ids = [
        PortalId::DungeonLeft,
        PortalId::DungeonCenter,
        PortalId::DungeonRight,
    ];

    let portal_types = [
        PortalType::Dungeon,
        PortalType::Dungeon,
        PortalType::Dungeon,
    ];

    let portal_colors = [
        Color::srgb(0.6, 0.3, 0.8), // Purple dungeon portals
        Color::srgb(0.8, 0.4, 1.0),
        Color::srgb(0.4, 0.2, 0.6),
    ];

    for (i, ((&position, &portal_id), &portal_type)) in portal_positions.iter()
        .zip(portal_ids.iter())
        .zip(portal_types.iter())
        .enumerate() {

        // Create portal callback (currently just logs)
        let portal_callback: InteractionCallback = Arc::new(move |_context| {
            info!("Portal {:?} ({:?}) activated", portal_id, portal_type);
        });

        // Create portal entity with different sizes for different types
        let (width, height) = match portal_type {
            PortalType::Dungeon => (80.0, 120.0),
        };

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(portal_colors[i])),
            Transform::from_translation(position),
            Cathedral,
            Portal {
                id: portal_id,
                portal_type,
                depth: 1,
                modifiers: Vec::new(),
            },
            Interactable::new(
                format!("portal_{:?}", portal_id),
                format!("{:?} Portal", portal_id),
                portal_callback
            ),
            InteractableHighlight::with_radius(1.4),
        ));
    }

    // Floor is now handled by the tilemap - no need for separate floor decoration

    // Add Cathedral title text - positioned above the tilemap area
    let tilemap_width = crate::world::tiles::TILEMAP_WIDTH as f32 * crate::world::tiles::TILE_SIZE;
    let title_x = tilemap_width / 2.0; // Center of tilemap width
    commands.spawn((
        Text2d::new("The Cathedral"),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(title_x, 50.0, 1.0)), // Above portals, centered on tilemap
        Cathedral,
    ));

    info!("Cathedral scene set up with three portals");
}

/// Initialize portal configurations with modifiers
pub fn initialize_portals(
    mut modifier_system: ResMut<ModifierSystem>,
    mut portal_query: Query<&mut Portal>,
    progression_state: Res<ProgressionState>,
) {
    let mut rng = rand::rng();

    // Get the current available depths
    let available_depths = progression_state.get_available_depths();
    let current_depth = available_depths.first().copied().unwrap_or(1);

    // Generate modifiers for the current depth
    modifier_system.generate_portal_modifiers(current_depth, &mut rng);

    // Update portal components with their modifiers
    for mut portal in portal_query.iter_mut() {
        portal.depth = current_depth;
        portal.modifiers = modifier_system.get_portal_modifiers(current_depth, portal.id);
    }

    info!("Portals initialized for depth {} with modifiers", current_depth);
}

/// Update portal display texts with current modifier information
pub fn update_portal_displays(
    portal_query: Query<&Portal>,
    mut display_query: Query<&mut Text2d, With<PortalDisplay>>,
    display_component_query: Query<&PortalDisplay>,
) {
    for mut text2d in display_query.iter_mut() {
        // Find the corresponding PortalDisplay component
        if let Some(display_component) = display_component_query.iter()
            .find(|_d| {
                // This is a bit hacky - we need to match the display component to the text
                // In a more robust system, we'd store entity IDs or use a better linking method
                true // For now, we'll update all displays
            }) {

            // Find the portal with the matching ID
            if let Some(portal) = portal_query.iter()
                .find(|p| p.id == display_component.portal_id) {

                let mut display_text = format!("Depth {}\n", portal.depth);

                if portal.modifiers.is_empty() {
                    display_text.push_str("No Modifiers");
                } else {
                    for modifier in &portal.modifiers {
                        display_text.push_str(&format!("• {}\n", modifier.display_name()));
                    }
                }

                **text2d = display_text;
            }
        }
    }
}



/// Handle portal interactions through the event system
pub fn handle_portal_interaction_events(
    mut interaction_events: EventReader<InteractionEvent>,
    mut portal_activation_events: EventWriter<crate::events::PortalActivationEvent>,
    portal_query: Query<&Portal>,
) {
    for event in interaction_events.read() {
        // Check if the interaction is with a portal (by checking if the entity has a Portal component)
        if let Ok(portal) = portal_query.get(event.target_entity) {
            info!("Portal interaction detected: {:?} -> depth {}", portal.id, portal.depth);

            // Send portal activation event instead of directly transitioning
            portal_activation_events.write(crate::events::PortalActivationEvent {
                portal_id: portal.id,
                depth: portal.depth,
                modifiers: portal.modifiers.iter().map(|m| m.display_name()).collect(),
            });

            info!("Portal activation event sent for {:?}", portal.id);
            return;
        }
    }
}

/// Handle portal activation and scene transitions
pub fn handle_portal_activation(
    mut events: EventReader<crate::events::PortalActivationEvent>,
    current_state: Res<State<crate::world::states::WorldState>>,
    mut next_state: ResMut<NextState<crate::world::states::WorldState>>,
    portals: Query<&super::components::Portal>,
) {
    use crate::world::states::WorldState;

    for event in events.read() {
        if matches!(current_state.get(), WorldState::Cathedral) {
            // Find the portal that was activated
            if let Some(portal) = portals.iter().find(|p| p.id == event.portal_id) {
                match portal.portal_type {
                    super::components::PortalType::Dungeon => {
                        info!("Cathedral: Activating dungeon portal {:?} for depth {}", portal.id, event.depth);
                        next_state.set(WorldState::Dungeon);
                    },
                }
            }
        }
    }
}
