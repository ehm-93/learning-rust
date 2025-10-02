use bevy::prelude::*;

use crate::{
    world::states::WorldState,
};

use super::components::{DungeonEntity, DungeonExitPortal};
use super::resources::DungeonState;

/// Set up the dungeon scene when entering
pub fn setup_dungeon_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    dungeon_state: Res<DungeonState>,
) {
    info!("Setting up Dungeon scene (depth {})", dungeon_state.depth);

    // Spawn camera
    commands.spawn((
        Camera2d,
        crate::components::MainCamera,
        DungeonEntity,
    ));

    // Spawn simple floor
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(400.0, 400.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.4, 0.3, 0.3))), // Dark red
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        DungeonEntity,
    ));

    // Add dungeon title text with depth
    commands.spawn((
        Text2d::new(format!("Dungeon - Depth {} - Combat Zone", dungeon_state.depth)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.3, 0.3)),
        Transform::from_translation(Vec3::new(0.0, 150.0, 1.0)),
        DungeonEntity,
    ));

    // Spawn exit portal to sanctuary
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(60.0, 80.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.8, 0.3))), // Green portal
        Transform::from_translation(Vec3::new(0.0, -150.0, 0.0)),
        DungeonExitPortal,
        DungeonEntity,
        crate::world::Interactable::new(
            "dungeon_exit_portal".to_string(),
            "Portal to Sanctuary".to_string(),
            std::sync::Arc::new(|_context| {
                info!("Dungeon exit portal activated");
            })
        ),
        crate::world::InteractableHighlight::with_radius(1.4),
    ));

    // Add portal label
    commands.spawn((
        Text2d::new("Portal to Sanctuary"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, -200.0, 1.0)),
        DungeonEntity,
    ));

    // Spawn player at dungeon entrance
    let player_entity = crate::player::PlayerSpawner::spawn_with_commands(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 100.0, 0.0), // Position player above center
    );

    // Mark player as a dungeon entity for cleanup
    commands.entity(player_entity).insert(DungeonEntity);

    info!("Dungeon scene setup complete");
}

/// Clean up dungeon scene when exiting
pub fn teardown_dungeon_scene(
    mut commands: Commands,
    dungeon_entities: Query<Entity, With<DungeonEntity>>,
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
    exit_portals: Query<Entity, With<DungeonExitPortal>>,
    mut next_state: ResMut<NextState<WorldState>>,
) {
    for event in interaction_events.read() {
        // Check if the interacted entity is a dungeon exit portal
        for portal_entity in exit_portals.iter() {
            if event.target_entity == portal_entity {
                info!("Dungeon: Portal to Sanctuary activated - transitioning");
                next_state.set(WorldState::Sanctuary);
                return;
            }
        }
    }
}
