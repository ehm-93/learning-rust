pub mod components;
pub mod resources;
pub mod systems;
pub mod scene;

use bevy::prelude::*;
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
) {
    info!("Entering Cathedral state - setting up scene");

    // Mark Cathedral as active
    cathedral_state.is_active = true;
    cathedral_state.current_depth = 1; // Default starting depth

    // Spawn camera
    commands.spawn((
        Camera2d,
        crate::components::MainCamera,
        components::Cathedral, // Tag for cleanup
    ));

    // Spawn player at cathedral center using PlayerSpawner (with higher Z to render above tilemap)
    let player_entity = crate::player::PlayerSpawner::spawn_with_commands(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 0.0, 1.0) // Centered in tilemap area, Z=1 to render above tilemap at Z=-1
    );

    // Add cathedral tag for cleanup
    commands.entity(player_entity).insert(components::Cathedral);

    // Spawn the tilemap for the Cathedral scene
    crate::world::mapgen::spawn_cathedral_tilemap(&mut commands, &asset_server);

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

    // Spawn Cathedral marker entity
    commands.spawn((
        components::Cathedral,
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        Visibility::Visible,
    ));

    // Create three dungeon portals
    let portal_positions = [
        Vec3::new(-200.0, 200.0, 0.0), // Left dungeon portal - positioned above tilemap
        Vec3::new(0.0, 200.0, 0.0),    // Center dungeon portal
        Vec3::new(200.0, 200.0, 0.0),  // Right dungeon portal
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
    commands.spawn((
        Text2d::new("The Cathedral"),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, 300.0, 1.0)), // Higher up to be above portals
        components::Cathedral,
    ));

    info!("Cathedral scene set up with three portals");
}

// Re-export commonly used types
pub use components::*;
