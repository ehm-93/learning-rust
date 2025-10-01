use bevy::prelude::*;

use crate::{
    world::states::WorldState,
};

use super::components::{SanctuaryEntity, SanctuaryExitPortal, SanctuaryDungeonPortal};

/// Set up the sanctuary scene when entering
pub fn setup_sanctuary_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sanctuary_state: Res<super::resources::SanctuaryState>,
) {
    info!("Setting up Sanctuary scene (depth {})", sanctuary_state.current_depth);

    // Spawn camera
    commands.spawn((
        Camera2d,
        crate::components::MainCamera,
        SanctuaryEntity,
    ));

    // Spawn simple floor
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(400.0, 400.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.9, 0.8))), // Light green
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        SanctuaryEntity,
    ));

    // Add sanctuary title text with depth
    commands.spawn((
        Text2d::new(format!("Sanctuary - Depth {} - Safe Zone", sanctuary_state.current_depth)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.2, 0.6, 0.2)),
        Transform::from_translation(Vec3::new(0.0, 150.0, 1.0)),
        SanctuaryEntity,
    ));

    // Create three dungeon portals
    let portal_positions = [
        Vec3::new(-200.0, 50.0, 0.0),  // Left dungeon portal
        Vec3::new(0.0, 50.0, 0.0),     // Center dungeon portal
        Vec3::new(200.0, 50.0, 0.0),   // Right dungeon portal
    ];

    let portal_colors = [
        Color::srgb(0.8, 0.3, 0.3), // Red dungeon portals
        Color::srgb(0.9, 0.4, 0.4),
        Color::srgb(0.7, 0.2, 0.2),
    ];

    let portal_labels = [
        "Dungeon Left",
        "Dungeon Center",
        "Dungeon Right",
    ];

    for (i, (&position, &color)) in portal_positions.iter().zip(portal_colors.iter()).enumerate() {
        // Spawn dungeon portal
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(60.0, 80.0))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(position),
            SanctuaryDungeonPortal,
            SanctuaryEntity,
            crate::world::Interactable::new(
                format!("sanctuary_dungeon_portal_{}", i),
                format!("Portal to {}", portal_labels[i]),
                std::sync::Arc::new(move |_context| {
                    info!("Sanctuary dungeon portal {} activated", i);
                })
            ),
            crate::world::InteractableHighlight::with_radius(1.4),
        ));

        // Add portal label
        commands.spawn((
            Text2d::new(portal_labels[i]),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(position + Vec3::new(0.0, -50.0, 1.0)),
            SanctuaryEntity,
        ));
    }

    // Spawn exit portal back to cathedral (moved to bottom)
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(60.0, 80.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 0.8))), // Blue portal
        Transform::from_translation(Vec3::new(0.0, -150.0, 0.0)),
        SanctuaryExitPortal,
        SanctuaryEntity,
        crate::world::Interactable::new(
            "sanctuary_exit_portal".to_string(),
            "Portal to Cathedral".to_string(),
            std::sync::Arc::new(|_context| {
                info!("Sanctuary exit portal activated");
            })
        ),
        crate::world::InteractableHighlight::with_radius(1.4),
    ));

    // Add cathedral portal label
    commands.spawn((
        Text2d::new("Portal to Cathedral"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, -200.0, 1.0)),
        SanctuaryEntity,
    ));

    // Spawn player at sanctuary entrance
    let player_entity = crate::player::PlayerSpawner::spawn_with_commands(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 100.0, 0.0), // Position player above center
    );

    // Mark player as a sanctuary entity for cleanup
    commands.entity(player_entity).insert(SanctuaryEntity);

    info!("Sanctuary scene setup complete");
}

/// Clean up sanctuary scene when exiting
pub fn teardown_sanctuary_scene(
    mut commands: Commands,
    sanctuary_entities: Query<Entity, With<SanctuaryEntity>>,
) {
    info!("Tearing down Sanctuary scene");

    // Despawn all sanctuary entities
    for entity in sanctuary_entities.iter() {
        commands.entity(entity).despawn();
    }

    info!("Sanctuary scene teardown complete");
}

/// Handle sanctuary portal interactions (both cathedral and dungeon portals)
pub fn handle_sanctuary_portal_interactions(
    mut interaction_events: EventReader<crate::world::InteractionEvent>,
    exit_portals: Query<Entity, With<SanctuaryExitPortal>>,
    dungeon_portals: Query<Entity, With<SanctuaryDungeonPortal>>,
    mut next_state: ResMut<NextState<WorldState>>,
    sanctuary_state: Res<super::resources::SanctuaryState>,
    mut dungeon_state: ResMut<crate::world::scenes::dungeon::resources::DungeonState>,
) {
    for event in interaction_events.read() {
        // Check if the interacted entity is a sanctuary exit portal (to cathedral)
        for portal_entity in exit_portals.iter() {
            if event.target_entity == portal_entity {
                info!("Sanctuary: Portal to Cathedral activated - transitioning");
                next_state.set(WorldState::Cathedral);
                return;
            }
        }

        // Check if the interacted entity is a sanctuary dungeon portal
        for portal_entity in dungeon_portals.iter() {
            if event.target_entity == portal_entity {
                let next_depth = sanctuary_state.next_dungeon_depth();
                info!("Sanctuary: Portal to Dungeon activated - transitioning to depth {}", next_depth);

                // Update dungeon state with the next depth
                dungeon_state.depth = next_depth;
                dungeon_state.cleared_rooms = 0; // Reset progress for new depth
                dungeon_state.is_completed = false;

                next_state.set(WorldState::Dungeon);
                return;
            }
        }
    }
}

/// System to update sanctuary depth when entering from dungeon
pub fn update_sanctuary_depth_on_enter(
    mut sanctuary_state: ResMut<super::resources::SanctuaryState>,
    dungeon_state: Res<crate::world::scenes::dungeon::resources::DungeonState>,
) {
    // Update sanctuary depth to match the current dungeon depth
    // This ensures sanctuary knows what depth the player has reached
    if sanctuary_state.current_depth < dungeon_state.depth {
        sanctuary_state.set_depth(dungeon_state.depth);
        info!("Sanctuary depth updated to {}", sanctuary_state.current_depth);
    }
}
