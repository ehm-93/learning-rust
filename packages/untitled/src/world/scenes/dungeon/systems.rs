use bevy::prelude::*;

use crate::world;
use crate::world::mapgen;
use crate::world::chunks;
use super::components;
use super::resources;

/// Set up the dungeon scene when entering
pub fn setup_dungeon_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut dungeon_state: ResMut<resources::DungeonState>,
    camera_zoom: Res<crate::player::resources::CameraZoom>,
) {
    info!("Setting up Dungeon scene (depth {})", dungeon_state.depth);

    // Create level based on dungeon state
    // Note: each macro cell is 0.5 chunks (16x16 meters)
    dungeon_state.macro_map = mapgen::freeform(
        world::DUNGEON_SIZE_M / world::METERS_PER_CHUNK * world::MACRO_PX_PER_CHUNK,
        dungeon_state.seed,
    );

    info!("Dungeon macro map generated with dimensions: {}x{}", dungeon_state.macro_map.len(), dungeon_state.macro_map[0].len());

    // Enable chunking in dungeon, procedural map
    commands.set_state(chunks::ChunkingState::Enabled);

    // Spawn camera
    let zoom_level = camera_zoom.level;
    commands.spawn((
        Camera2d,
        Transform {
            scale: Vec3::splat(zoom_level),
            ..default()
        },
        crate::components::MainCamera,
        components::Dungeon, // Tag for cleanup
    ));

    let spawn_pos = (world::DUNGEON_SIZE_PX / 2, world::DUNGEON_SIZE_PX / 2);

    // Spawn exit portal to sanctuary
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(60.0, 80.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.8, 0.3))), // Green portal
        Transform::from_translation(Vec3::new(spawn_pos.0 as f32, spawn_pos.1 as f32 - 150.0, 0.0)),
        components::DungeonExitPortal,
        components::Dungeon,
        crate::world::Interactable::new(
            "dungeon_exit_portal".to_string(),
            "Portal to Sanctuary".to_string(),
            |_context| {
                info!("Dungeon exit portal activated");
            }
        ),
        crate::world::InteractableHighlight::with_radius(1.4),
    ));

    // Spawn player at dungeon entrance (at spawn room center)
    commands.spawn((
        crate::player::components::PlayerBundle::new(
            &mut meshes,
            &mut materials,
            Vec3::new(spawn_pos.0 as f32, spawn_pos.1 as f32, 0.0),
        ),
        components::Dungeon,
    ));

    info!("Dungeon scene setup complete");
}

/// Clean up dungeon scene when exiting
pub fn teardown_dungeon_scene(
    mut commands: Commands,
    dungeon_entities: Query<Entity, With<components::Dungeon>>,
) {
    info!("Tearing down Dungeon scene");

    // Despawn all dungeon entities
    for entity in dungeon_entities.iter() {
        commands.entity(entity).despawn();
    }

    info!("Dungeon scene teardown complete");
}

/// Handle dungeon exit portal interactions
pub fn handle_dungeon_portal_interactions(
    mut interaction_events: EventReader<crate::world::InteractionEvent>,
    exit_portals: Query<Entity, With<components::DungeonExitPortal>>,
    mut next_state: ResMut<NextState<world::WorldState>>,
) {
    for event in interaction_events.read() {
        // Check if the interacted entity is a dungeon exit portal
        for portal_entity in exit_portals.iter() {
            if event.target_entity == portal_entity {
                info!("Dungeon: Portal to Sanctuary activated - transitioning");
                next_state.set(world::WorldState::Sanctuary);
                return;
            }
        }
    }
}
