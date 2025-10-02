pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::world::states::WorldState;

/// Cathedral plugin that manages the central hub and dungeon descent system
pub struct CathedralPlugin;

impl Plugin for CathedralPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add resources
            .init_resource::<resources::ModifierSystem>()
            .init_resource::<resources::ProgressionState>()
            .init_resource::<resources::CathedralState>()
            .init_resource::<crate::world::states::CathedralConfig>()

            // Scene lifecycle systems
            .add_systems(OnEnter(WorldState::Cathedral), setup_cathedral_scene)
            .add_systems(OnExit(WorldState::Cathedral), teardown_cathedral_scene)
            .add_systems(
                Update,
                (
                    systems::handle_portal_interaction_events.run_if(in_state(WorldState::Cathedral)),
                    systems::handle_portal_activation.run_if(in_state(WorldState::Cathedral)),
                    systems::update_portal_displays.run_if(in_state(WorldState::Cathedral)),
                )
            );
    }
}

/// System to setup the cathedral scene when entering WorldState::Cathedral
fn setup_cathedral_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut modifier_system: ResMut<resources::ModifierSystem>,
    mut cathedral_state: ResMut<resources::CathedralState>,
    progression_state: Res<resources::ProgressionState>,
    asset_server: Res<AssetServer>,
    camera_zoom: Res<crate::player::resources::CameraZoom>,
) {
    info!("Entering Cathedral state - setting up scene");

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
        components::Cathedral, // Tag for cleanup
    ));

    // Spawn player in the center of the tilemap area (now in positive quadrant)
    let tilemap_center_x = (crate::world::mapgen::TILEMAP_WIDTH as f32 * crate::world::mapgen::TILE_SIZE) / 2.0;
    let tilemap_center_y = (crate::world::mapgen::TILEMAP_HEIGHT as f32 * crate::world::mapgen::TILE_SIZE) / 2.0;
    let player_entity = commands.spawn(
        crate::player::components::PlayerBundle::new(
            &mut meshes,
            &mut materials,
            Vec3::new(tilemap_center_x, tilemap_center_y, 1.0) // Center of tilemap, Z=1 to render above tilemap at Z=-1
        ),
    ).id();

    // Add cathedral tag for cleanup
    commands.entity(player_entity).insert(components::Cathedral);

    // Create tilemap directly in cathedral scene
    let texture_handle: Handle<Image> = asset_server.load("sprites/tiles.png");

    let map_size = TilemapSize {
        x: crate::world::mapgen::TILEMAP_WIDTH as u32,
        y: crate::world::mapgen::TILEMAP_HEIGHT as u32
    };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Generate the test tilemap pattern
    let tilemap_data = crate::world::mapgen::generate_test_tilemap();

    // Store wall positions for collision spawning
    let mut wall_positions = Vec::new();

    // Spawn each tile
    for x in 0..crate::world::mapgen::TILEMAP_WIDTH {
        for y in 0..crate::world::mapgen::TILEMAP_HEIGHT {
            let tile_pos = TilePos { x: x as u32, y: y as u32 };
            let tile_type = tilemap_data[y as usize][x as usize];

            // Set texture index based on tile type
            let texture_index = match tile_type {
                crate::world::mapgen::TileType::Floor => TileTextureIndex(0),
                crate::world::mapgen::TileType::Wall => TileTextureIndex(1),
            };

            let tile_entity = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index,
                ..Default::default()
            }).id();

            // Mark wall tiles and collect positions for collision
            if tile_type == crate::world::mapgen::TileType::Wall {
                commands.entity(tile_entity).insert(crate::world::mapgen::WallTile);
                wall_positions.push((x, y));
            }

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Now spawn separate collider entities for each wall
    for (x, y) in wall_positions {
        let world_x = x as f32 * crate::world::mapgen::TILE_SIZE;
        let world_y = y as f32 * crate::world::mapgen::TILE_SIZE;

        commands.spawn((
            crate::world::mapgen::WallTile,
            Collider::cuboid(
                crate::world::mapgen::TILE_SIZE / 2.0,
                crate::world::mapgen::TILE_SIZE / 2.0
            ),
            RigidBody::Fixed,
            Transform::from_xyz(world_x, world_y, -1.0),
            Visibility::Hidden, // Invisible collider - visual handled by tile
            components::Cathedral, // Tag for cleanup
        ));
    }

    // Define tilemap properties - everything uses 16x16 consistently
    let grid_size = TilemapGridSize {
        x: crate::world::mapgen::TILE_SIZE, // 16.0
        y: crate::world::mapgen::TILE_SIZE
    };
    let map_type = TilemapType::Square;
    let tile_size = TilemapTileSize {
        x: crate::world::mapgen::TILE_SIZE, // 16.0 - matches sprite atlas
        y: crate::world::mapgen::TILE_SIZE
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
    .insert(crate::world::mapgen::GameTilemap)
    .insert(components::Cathedral); // Tag for cleanup

    info!("Successfully spawned Cathedral tilemap with {} tiles",
          crate::world::mapgen::TILEMAP_WIDTH * crate::world::mapgen::TILEMAP_HEIGHT);

    setup_cathedral_entities(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut modifier_system,
        &progression_state,
    );
}

/// System to cleanup the cathedral scene when exiting WorldState::Cathedral
fn teardown_cathedral_scene(
    mut commands: Commands,
    mut cathedral_state: ResMut<resources::CathedralState>,
    cathedral_entities: Query<Entity, With<components::Cathedral>>,
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
    _modifier_system: &mut ResMut<resources::ModifierSystem>,
    _progression_state: &Res<resources::ProgressionState>,
) {
    use std::sync::Arc;
    use crate::world::{Interactable, InteractableHighlight, InteractionCallback};

    // Spawn Cathedral marker entity at tilemap center
    let tilemap_width = crate::world::mapgen::TILEMAP_WIDTH as f32 * crate::world::mapgen::TILE_SIZE;
    let tilemap_height = crate::world::mapgen::TILEMAP_HEIGHT as f32 * crate::world::mapgen::TILE_SIZE;
    commands.spawn((
        components::Cathedral,
        Transform::from_translation(Vec3::new(tilemap_width / 2.0, tilemap_height / 2.0, -1.0)),
        Visibility::Visible,
    ));

    // Create three dungeon portals positioned in the center of the 128m×128m box
    let box_center_x = (crate::world::mapgen::TILEMAP_WIDTH as f32 * crate::world::mapgen::TILE_SIZE) / 2.0;
    let box_center_y = (crate::world::mapgen::TILEMAP_HEIGHT as f32 * crate::world::mapgen::TILE_SIZE) / 2.0;
    let portal_positions = [
        Vec3::new(box_center_x - 200.0, box_center_y, 0.0), // Left portal
        Vec3::new(box_center_x, box_center_y, 0.0),         // Center portal
        Vec3::new(box_center_x + 200.0, box_center_y, 0.0), // Right portal
    ];

    let portal_ids = [
        components::PortalId::DungeonLeft,
        components::PortalId::DungeonCenter,
        components::PortalId::DungeonRight,
    ];

    let portal_types = [
        components::PortalType::Dungeon,
        components::PortalType::Dungeon,
        components::PortalType::Dungeon,
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
            components::PortalType::Dungeon => (80.0, 120.0),
        };

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(portal_colors[i])),
            Transform::from_translation(position),
            components::Cathedral,
            components::Portal {
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
    let tilemap_width = crate::world::mapgen::TILEMAP_WIDTH as f32 * crate::world::mapgen::TILE_SIZE;
    let title_x = tilemap_width / 2.0; // Center of tilemap width
    commands.spawn((
        Text2d::new("The Cathedral"),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(title_x, 50.0, 1.0)), // Above portals, centered on tilemap
        components::Cathedral,
    ));

    info!("Cathedral scene set up with three portals");
}



// Re-export commonly used types
pub use components::*;
