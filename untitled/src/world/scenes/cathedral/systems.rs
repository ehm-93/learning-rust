use bevy::prelude::*;
use std::sync::Arc;
use crate::{
    components::*,
    resources::*,
    world::{Interactable, InteractionEvent, InteractableHighlight, InteractionCallback},
    player::Player,
};

use super::{
    components::*,
    resources::*,
};

/// Setup the Cathedral scene with three portal archways
pub fn setup_cathedral_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cathedral_state: ResMut<CathedralState>,
) {
    // Mark Cathedral as active
    cathedral_state.is_active = true;
    cathedral_state.current_depth = 1; // Default starting depth

    // Spawn Cathedral marker entity
    commands.spawn((
        Cathedral,
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        Visibility::Visible,
    ));

    // Create three portal archways
    let portal_positions = [
        Vec3::new(-300.0, 100.0, 0.0), // Left portal
        Vec3::new(0.0, 100.0, 0.0),    // Center portal
        Vec3::new(300.0, 100.0, 0.0),  // Right portal
    ];

    let portal_ids = [PortalId::Left, PortalId::Center, PortalId::Right];
    let portal_colors = [Color::srgb(0.6, 0.3, 0.8), Color::srgb(0.8, 0.4, 1.0), Color::srgb(0.4, 0.2, 0.6)];

    for (i, (&position, &portal_id)) in portal_positions.iter().zip(portal_ids.iter()).enumerate() {
                // Create portal callback for transitions
        let portal_callback: InteractionCallback = Arc::new(move |_context| {
            info!("Portal {:?} activated - transitioning to dungeon depth 1", portal_id);
            info!("For now, portals just log this message. Actual dungeon generation will be implemented soon!");
            // The actual transition logic will be handled by the portal_interaction_events system
        });

        // Create portal entity
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(80.0, 120.0))),
            MeshMaterial2d(materials.add(portal_colors[i])),
            Transform::from_translation(position),
            Cathedral,
            Portal {
                id: portal_id,
                depth: 1, // Will be updated by initialize_portals
                modifiers: Vec::new(), // Will be populated by initialize_portals
            },
            Interactable::new(
                format!("portal_{:?}", portal_id), // id
                format!("{:?} Portal", portal_id), // display name
                portal_callback
            ),
            InteractableHighlight::with_radius(1.4), // Use default colors with custom radius
        ));

        // Create portal display text (positioned below the portal)
        let text_position = position + Vec3::new(0.0, -120.0, 1.0);
        commands.spawn((
            Text2d::new("Portal Loading..."),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(text_position),
            PortalDisplay { portal_id },
        ));
    }

    // Add some basic Cathedral decoration (simple floor)
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(800.0, 100.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::from_translation(Vec3::new(0.0, -200.0, -1.0)),
        Cathedral,
    ));

    // Add Cathedral title text
    commands.spawn((
        Text2d::new("The Cathedral"),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, 250.0, 1.0)),
        Cathedral,
    ));

    info!("Cathedral scene initialized with three portals");
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

/// Handle player interaction with portals
pub fn handle_portal_interaction(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    portal_query: Query<&Portal>,
    _player_query: Query<Entity, With<Player>>,
    cathedral_query: Query<Entity, With<Cathedral>>,
    mut cathedral_state: ResMut<CathedralState>,
    mut game_mode: ResMut<GameMode>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Simple keyboard controls for portal selection (will be replaced with mouse/UI later)
    let selected_portal = if keys.just_pressed(KeyCode::Digit1) {
        Some(PortalId::Left)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(PortalId::Center)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(PortalId::Right)
    } else {
        None
    };

    if let Some(portal_id) = selected_portal {
        // Find the selected portal
        if let Some(portal) = portal_query.iter().find(|p| p.id == portal_id) {
            info!("Player selected {:?} portal for depth {} with modifiers: {:?}",
                  portal_id, portal.depth, portal.modifiers);

            // Transition from Cathedral to dungeon
            cathedral_state.is_active = false;
            *game_mode = GameMode::Dungeon;

            // Despawn Cathedral entities
            for entity in cathedral_query.iter() {
                commands.entity(entity).despawn();
            }

            // For now, just generate a basic dungeon (will be enhanced with modifiers later)
            // TODO: Apply portal modifiers to dungeon generation
            crate::world::dungeon::generate_dungeon_rooms(&mut commands, &mut meshes, &mut materials, portal.depth);

            info!("Transitioned to dungeon at depth {}", portal.depth);
        }
    }

    // Handle return to Cathedral (R key for now)
    if keys.just_pressed(KeyCode::KeyR) && !cathedral_state.is_active {
        info!("Returning to Cathedral");

        // Clear the current dungeon
        // TODO: Implement proper dungeon cleanup system

        // Re-setup Cathedral
        cathedral_state.is_active = true;
        *game_mode = GameMode::Cathedral;

        // Re-spawn Cathedral (this is a simple approach, could be optimized)
        setup_cathedral_scene(commands, meshes, materials, cathedral_state);
    }
}

/// Handle portal interactions through the event system
pub fn handle_portal_interaction_events(
    mut commands: Commands,
    mut interaction_events: EventReader<InteractionEvent>,
    portal_query: Query<&Portal>,
    cathedral_query: Query<Entity, With<Cathedral>>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<Portal>)>,
    mut cathedral_state: ResMut<CathedralState>,
    mut game_mode: ResMut<GameMode>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in interaction_events.read() {
        // Check if the interaction is with a portal (by checking if the entity has a Portal component)
        if let Ok(portal) = portal_query.get(event.target_entity) {
            info!("Portal interaction detected: {:?} -> depth {}", portal.id, portal.depth);

            // Move player to a different position
            if let Ok(mut player_transform) = player_query.single_mut() {
                player_transform.translation = Vec3::new(0.0, -200.0, 0.0); // Move player down
                info!("Player moved to dungeon position");
            }

            // Transition from Cathedral to dungeon
            cathedral_state.is_active = false;
            *game_mode = GameMode::Dungeon;

            // Despawn Cathedral entities
            for entity in cathedral_query.iter() {
                commands.entity(entity).despawn();
            }

            // Create a simple "dungeon" scene instead of the complex generator
            spawn_simple_dungeon(&mut commands, &mut meshes, &mut materials);

            info!("Transitioned to simple dungeon at depth {} via interaction system", portal.depth);
            return;
        }
    }
}

/// Create a simple dungeon scene for testing
fn spawn_simple_dungeon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    info!("Spawning simple test dungeon");

    // Create a simple floor
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(600.0, 400.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.3, 0.4))), // Dark blue-gray floor
        Transform::from_translation(Vec3::new(0.0, -300.0, -1.0)),
    ));

    // Create some simple walls
    let wall_color = Color::srgb(0.4, 0.4, 0.5);

    // Left wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(20.0, 400.0))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_translation(Vec3::new(-300.0, -300.0, 0.0)),
    ));

    // Right wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(20.0, 400.0))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_translation(Vec3::new(300.0, -300.0, 0.0)),
    ));

    // Back wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(600.0, 20.0))),
        MeshMaterial2d(materials.add(wall_color)),
        Transform::from_translation(Vec3::new(0.0, -500.0, 0.0)),
    ));

    // Add a simple "return portal"
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(60.0, 80.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.8, 0.2))), // Yellow return portal
        Transform::from_translation(Vec3::new(0.0, -400.0, 0.0)),
    ));

    info!("Simple test dungeon spawned successfully");
}
