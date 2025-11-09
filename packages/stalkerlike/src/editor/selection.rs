use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::{EditorCamera, EditorEntity};
use super::placement::PlacementState;

/// Resource tracking the currently selected entity
#[derive(Resource, Default)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
}

/// Marker component for selected entities
#[derive(Component)]
pub struct Selected;

/// Handle entity selection via raycasting
pub fn handle_selection(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform, &EditorCamera)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut selected: ResMut<SelectedEntity>,
    mut commands: Commands,
    entity_query: Query<(Entity, &GlobalTransform, &ViewVisibility), With<EditorEntity>>,
    selected_query: Query<Entity, With<Selected>>,
    placement_state: Res<PlacementState>,
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Mesh3d>,
) {
    // Only select when mouse is unlocked and not in placement mode
    let Ok((camera, camera_transform, editor_camera)) = camera_query.get_single() else {
        return;
    };

    if editor_camera.mouse_locked || placement_state.active {
        return;
    }

    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Cast ray from camera through cursor
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Find closest entity hit by ray
    let mut closest_hit: Option<(Entity, f32)> = None;

    for (entity, transform, visibility) in entity_query.iter() {
        if !visibility.get() {
            continue;
        }

        // Simple sphere-based intersection test (approximation)
        let entity_pos = transform.translation();
        let to_entity = entity_pos - ray.origin;
        let projection = to_entity.dot(*ray.direction);

        if projection < 0.0 {
            continue; // Behind camera
        }

        let closest_point = ray.origin + *ray.direction * projection;
        let distance_to_ray = (entity_pos - closest_point).length();

        // Use a generous radius for selection (1.0 units)
        let selection_radius = 1.0;

        if distance_to_ray < selection_radius {
            let distance = projection;
            if closest_hit.is_none() || distance < closest_hit.unwrap().1 {
                closest_hit = Some((entity, distance));
            }
        }
    }

    // Clear previous selection
    for entity in selected_query.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    // Select the clicked entity if any
    if let Some((entity, _)) = closest_hit {
        selected.entity = Some(entity);
        commands.entity(entity).insert(Selected);
        info!("Selected entity: {:?}", entity);
    } else {
        // Clicked on empty space - deselect
        selected.entity = None;
        info!("Deselected");
    }
}

/// Handle deselection (ESC key)
pub fn handle_deselection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedEntity>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    placement_state: Res<PlacementState>,
) {
    // Don't deselect if in placement mode (ESC cancels placement instead)
    if placement_state.active {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) && selected.entity.is_some() {
        // Clear selection
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        selected.entity = None;
        info!("Deselected");
    }
}

/// Add visual outline to selected entities
pub fn highlight_selected(
    mut commands: Commands,
    selected_query: Query<Entity, (With<Selected>, Without<Outline>)>,
) {
    for entity in selected_query.iter() {
        commands.entity(entity).insert(Outline {
            color: Color::srgb(1.0, 0.8, 0.0), // Yellow outline
            offset: Val::Px(2.0),
            width: Val::Px(3.0),
        });
    }
}

/// Remove outline from deselected entities
pub fn remove_outline_from_deselected(
    mut commands: Commands,
    outline_query: Query<Entity, (With<Outline>, Without<Selected>)>,
) {
    for entity in outline_query.iter() {
        commands.entity(entity).remove::<Outline>();
    }
}
