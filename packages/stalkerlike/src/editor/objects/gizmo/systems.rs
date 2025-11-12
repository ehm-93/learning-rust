//! Gizmo system functions
//!
//! This module contains all the systems that operate on gizmo entities:
//! - `toggle_transform_mode`: Handle keyboard input to cycle gizmo modes
//! - `spawn_gizmo`: Create gizmo visualization when entity selected
//! - `despawn_gizmo`: Clean up gizmo when entity deselected
//! - `update_gizmo_position`: Update gizmo transform to follow selection
//! - `on_gizmo_drag`: Handle drag events to transform selected objects
//! - `on_gizmo_hover`: Visual feedback when hovering over gizmo handles
//! - `on_gizmo_hover_end`: Reset visual feedback when hover ends
//!
//! # Gizmo Lifecycle
//!
//! - `spawn_gizmo()` triggers on OnAdd<Selected> - creates gizmo when entity selected
//! - `update_gizmo_position()` runs every frame - syncs gizmo to selected object
//! - `despawn_gizmo()` triggers on OnRemove<Selected> - cleans up when deselected
//! - Entity-specific observers (on_gizmo_*) handle all interaction events:
//!   * on_gizmo_drag: Update transform during Drag
//!   * on_gizmo_hover/on_gizmo_hover_end: Visual feedback on Over/Out

use bevy::prelude::*;
use bevy::picking::events::{Pointer, Drag, Over, Out};

use crate::editor::core::materials::GizmoMaterial;
use crate::editor::objects::selection::{SelectionSet, Selected};
use crate::editor::viewport::grid::GridConfig;

use super::types::{GizmoState, TransformMode, TransformOrientation, GizmoAxis, GizmoHandle, GizmoRoot};

/// Toggle transform mode with F key (forward) and Shift+F (backward)
pub fn toggle_transform_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gizmo_state: ResMut<GizmoState>,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if keyboard.just_pressed(KeyCode::KeyF) {
        gizmo_state.mode = if shift {
            // Shift+F: cycle backward
            match gizmo_state.mode {
                TransformMode::Translate => TransformMode::Scale,
                TransformMode::Rotate => TransformMode::Translate,
                TransformMode::Scale => TransformMode::Rotate,
            }
        } else {
            // F: cycle forward
            match gizmo_state.mode {
                TransformMode::Translate => TransformMode::Rotate,
                TransformMode::Rotate => TransformMode::Scale,
                TransformMode::Scale => TransformMode::Translate,
            }
        };
    }

    // Toggle orientation with O key
    if keyboard.just_pressed(KeyCode::KeyO) {
        gizmo_state.orientation = match gizmo_state.orientation {
            TransformOrientation::Global => TransformOrientation::Local,
            TransformOrientation::Local => TransformOrientation::Global,
        };
        info!("Transform orientation: {:?}", gizmo_state.orientation);
    }
}

/// Spawn gizmo handles at the selected object's position
///
/// Observers are attached directly to each gizmo handle entity for efficient
/// event handling. This is more performant than global observers since events
/// only trigger on the specific entity being interacted with.
///
/// This system runs when the Selected component is added to an entity.
pub fn spawn_gizmo(
    trigger: Trigger<OnAdd, Selected>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut gizmo_materials: ResMut<Assets<GizmoMaterial>>,
    transform_query: Query<&Transform>,
) {
    let selected_entity = trigger.target();

    let Ok(transform) = transform_query.get(selected_entity) else {
        return;
    };

    let position = transform.translation;

    // Arrow dimensions (Blender-style)
    let arrow_length = 1.0;      // Length of arrow shaft
    let arrow_radius = 0.02;      // Thickness of arrow shaft
    let cone_height = 0.2;        // Height of arrowhead cone
    let cone_radius = 0.05;       // Radius of arrowhead cone

    // Create root entity for gizmo (at selected object's position)
    let root = commands.spawn((
        GizmoRoot,
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    // Create arrow meshes
    let shaft_mesh = meshes.add(Cylinder::new(arrow_radius, arrow_length));
    let cone_mesh = meshes.add(Cone {
        radius: cone_radius,
        height: cone_height,
    });

    // Create thin line mesh for axis lines
    let line_mesh = meshes.add(Cylinder::new(arrow_radius * 0.5, arrow_length));

    // X-axis arrow (red) - points right
    // Axis line from center to handle
    let x_line = commands.spawn((
        Mesh3d(line_mesh.clone()),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(1.0, 0.0, 0.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(arrow_length / 2.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let x_shaft = commands.spawn((
        Mesh3d(shaft_mesh.clone()),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(1.0, 0.0, 0.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(arrow_length / 2.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let x_cone = commands.spawn((
        Mesh3d(cone_mesh.clone()),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(1.0, 0.0, 0.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(arrow_length + cone_height / 2.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let x_handle = commands.spawn((
        GizmoHandle,
        GizmoAxis::X,
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Pickable::default(),
    ))
    .observe(on_gizmo_hover)
    .observe(on_gizmo_hover_end)
    .observe(on_gizmo_drag)
    .add_children(&[x_line, x_shaft, x_cone])
    .id();

    // Y-axis arrow (green) - points up
    // Axis line from center to handle
    let y_line = commands.spawn((
        Mesh3d(line_mesh.clone()),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(0.0, 1.0, 0.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(0.0, arrow_length / 2.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let y_shaft = commands.spawn((
        Mesh3d(shaft_mesh.clone()),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(0.0, 1.0, 0.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(0.0, arrow_length / 2.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let y_cone = commands.spawn((
        Mesh3d(cone_mesh.clone()),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(0.0, 1.0, 0.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(0.0, arrow_length + cone_height / 2.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let y_handle = commands.spawn((
        GizmoHandle,
        GizmoAxis::Y,
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Pickable::default(),
    ))
    .observe(on_gizmo_hover)
    .observe(on_gizmo_hover_end)
    .observe(on_gizmo_drag)
    .add_children(&[y_line, y_shaft, y_cone])
    .id();

    // Z-axis arrow (blue) - points forward
    // Axis line from center to handle
    let z_line = commands.spawn((
        Mesh3d(line_mesh),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(0.0, 0.0, 1.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(0.0, 0.0, arrow_length / 2.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let z_shaft = commands.spawn((
        Mesh3d(shaft_mesh),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(0.0, 0.0, 1.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(0.0, 0.0, arrow_length / 2.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let z_cone = commands.spawn((
        Mesh3d(cone_mesh),
        MeshMaterial3d(gizmo_materials.add(GizmoMaterial::new(
            LinearRgba::rgb(0.0, 0.0, 1.0),
            LinearRgba::rgb(0.0, 0.0, 0.0),
        ))),
        Transform::from_xyz(0.0, 0.0, arrow_length + cone_height / 2.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    let z_handle = commands.spawn((
        GizmoHandle,
        GizmoAxis::Z,
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Pickable::default(),
    ))
    .observe(on_gizmo_hover)
    .observe(on_gizmo_hover_end)
    .observe(on_gizmo_drag)
    .add_children(&[z_line, z_shaft, z_cone])
    .id();

    // Parent handles to root
    commands.entity(root).add_children(&[x_handle, y_handle, z_handle]);
}

/// Despawn gizmo handles when entity is deselected
///
/// This system runs when the Selected component is removed from an entity.
pub fn despawn_gizmo(
    _trigger: Trigger<OnRemove, Selected>,
    mut commands: Commands,
    gizmo_query: Query<Entity, With<GizmoRoot>>,
) {
    // Remove all gizmo entities
    for entity in gizmo_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Update gizmo position, rotation, and scale to follow selected object(s)
/// For multi-select, positions at the center of all selected objects' bounding box
/// Scale is adjusted based on camera distance to maintain constant screen-space size
pub fn update_gizmo_position(
    selection: Res<SelectionSet>,
    selected_query: Query<&Transform, With<Selected>>,
    mut gizmo_query: Query<&mut Transform, (With<GizmoRoot>, Without<Selected>)>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    gizmo_state: Res<GizmoState>,
) {
    if selection.is_empty() {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // Calculate the center position of all selected entities
    let mut center = Vec3::ZERO;
    let mut count = 0;

    for entity in &selection.entities {
        if let Ok(transform) = selected_query.get(*entity) {
            center += transform.translation;
            count += 1;
        }
    }

    if count == 0 {
        return;
    }

    center /= count as f32;

    // Calculate distance from camera to gizmo center
    let distance = (camera_transform.translation() - center).length();

    // Scale based on distance to maintain constant screen-space size
    // The 0.15 factor controls the apparent size - tune as needed
    let scale_factor = distance * 0.15;

    for mut gizmo_transform in gizmo_query.iter_mut() {
        gizmo_transform.translation = center;

        // Apply rotation based on orientation mode
        // For multi-select, always use Global orientation (Local would be ambiguous)
        gizmo_transform.rotation = if selection.len() > 1 {
            Quat::IDENTITY // Multi-select always uses Global orientation
        } else if let Some(first_entity) = selection.first() {
            if let Ok(first_transform) = selected_query.get(first_entity) {
                match gizmo_state.orientation {
                    TransformOrientation::Global => Quat::IDENTITY,
                    TransformOrientation::Local => first_transform.rotation,
                }
            } else {
                Quat::IDENTITY
            }
        } else {
            Quat::IDENTITY
        };

        gizmo_transform.scale = Vec3::splat(scale_factor);
    }
}

/// Handle drag on gizmo handle (entity observer) - fires continuously while dragging
/// For multi-select, applies transform to all selected entities
pub fn on_gizmo_drag(
    drag: Trigger<Pointer<Drag>>,
    gizmo_state: Res<GizmoState>,
    handle_query: Query<&GizmoAxis, With<GizmoHandle>>,
    selection: Res<SelectionSet>,
    mut selected_query: Query<&mut Transform, With<Selected>>,
    grid_config: Res<GridConfig>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gizmo_query: Query<&Transform, (With<GizmoRoot>, Without<Selected>)>,
) {
    let handle_entity = drag.target();

    // Get the axis of the handle being dragged
    let Ok(axis) = handle_query.get(handle_entity) else {
        return;
    };

    if selection.is_empty() {
        return;
    }

    // Get drag delta from the event
    let delta = drag.event().delta;

    // Get camera for calculating drag scale
    let Ok((_camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Get gizmo position for distance calculation
    let Ok(gizmo_transform) = gizmo_query.single() else {
        return;
    };

    // Calculate drag scale based on distance from camera (farther = larger movements)
    let distance = (camera_transform.translation() - gizmo_transform.translation).length();
    let base_drag_scale = distance * 0.001; // Reduced from 0.002 for slower default speed

    // Apply speed modifiers based on keyboard input
    // Note: Ctrl is also used for multi-select, but during drag it modifies speed
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let speed_multiplier = if ctrl {
        0.25 // 1/4 speed when holding Ctrl
    } else if shift {
        4.0  // 4x speed when holding Shift
    } else {
        1.0  // Normal speed
    };

    let drag_scale = base_drag_scale * speed_multiplier;

    // Apply drag to ALL selected entities
    for entity in &selection.entities {
        let Ok(mut transform) = selected_query.get_mut(*entity) else {
            continue;
        };

        // Apply drag based on current mode and axis
        match gizmo_state.mode {
            TransformMode::Translate => {
                // Build movement vector in the gizmo's space
                let mut movement = Vec3::ZERO;
                match axis {
                    GizmoAxis::X => movement.x = delta.x * drag_scale,
                    GizmoAxis::Y => movement.y = -delta.y * drag_scale, // Invert Y for intuitive up/down
                    GizmoAxis::Z => movement.z = delta.y * drag_scale, // Use vertical drag for Z (forward/back)
                }

                // Transform movement based on orientation mode
                // For multi-select, always use Global (Local would be ambiguous)
                let world_movement = if selection.len() > 1 {
                    movement // Multi-select always in world space
                } else {
                    match gizmo_state.orientation {
                        TransformOrientation::Global => movement, // Already in world space
                        TransformOrientation::Local => transform.rotation * movement, // Rotate to local space
                    }
                };

                // Apply movement
                transform.translation += world_movement;

                // Apply grid snapping if enabled (in world space)
                if grid_config.snap_enabled {
                    let spacing = grid_config.spacing;
                    match axis {
                        GizmoAxis::X => {
                            if selection.len() == 1 || gizmo_state.orientation == TransformOrientation::Global {
                                transform.translation.x = (transform.translation.x / spacing).round() * spacing;
                            }
                        }
                        GizmoAxis::Y => {
                            if selection.len() == 1 || gizmo_state.orientation == TransformOrientation::Global {
                                transform.translation.y = (transform.translation.y / spacing).round() * spacing;
                            }
                        }
                        GizmoAxis::Z => {
                            if selection.len() == 1 || gizmo_state.orientation == TransformOrientation::Global {
                                transform.translation.z = (transform.translation.z / spacing).round() * spacing;
                            }
                        }
                    }
                }
            }
            TransformMode::Rotate => {
                // Rotation: use drag distance as angle delta
                let angle_delta = delta.length() * drag_scale * if delta.x + delta.y < 0.0 { -1.0 } else { 1.0 };

                // Build rotation quaternion around the selected axis
                let axis_rotation = match axis {
                    GizmoAxis::X => Quat::from_rotation_x(angle_delta),
                    GizmoAxis::Y => Quat::from_rotation_y(angle_delta),
                    GizmoAxis::Z => Quat::from_rotation_z(angle_delta),
                };

                // Apply rotation based on orientation mode
                // For multi-select, always use Global
                transform.rotation = if selection.len() > 1 {
                    axis_rotation * transform.rotation // Multi-select always Global
                } else {
                    match gizmo_state.orientation {
                        TransformOrientation::Global => {
                            // Global: rotate around world axis
                            axis_rotation * transform.rotation
                        }
                        TransformOrientation::Local => {
                            // Local: rotate around object's local axis
                            transform.rotation * axis_rotation
                        }
                    }
                };

                // Apply rotation snapping if enabled
                if grid_config.snap_enabled {
                    let snap_angle = 15.0_f32.to_radians();
                    let mut euler = transform.rotation.to_euler(EulerRot::XYZ);

                    match axis {
                        GizmoAxis::X => euler.0 = (euler.0 / snap_angle).round() * snap_angle,
                        GizmoAxis::Y => euler.1 = (euler.1 / snap_angle).round() * snap_angle,
                        GizmoAxis::Z => euler.2 = (euler.2 / snap_angle).round() * snap_angle,
                    }

                    transform.rotation = Quat::from_euler(EulerRot::XYZ, euler.0, euler.1, euler.2);
                }
            }
            TransformMode::Scale => {
                let scale_delta = -delta.y * drag_scale;

                match axis {
                    GizmoAxis::X => transform.scale.x = (transform.scale.x + scale_delta).max(0.01),
                    GizmoAxis::Y => transform.scale.y = (transform.scale.y + scale_delta).max(0.01),
                    GizmoAxis::Z => transform.scale.z = (transform.scale.z + scale_delta).max(0.01),
                }
            }
        }
    }
}

/// Highlight gizmo handle on hover (entity observer)
pub fn on_gizmo_hover(
    trigger: Trigger<Pointer<Over>>,
    handle_query: Query<&Children, With<GizmoHandle>>,
    material_query: Query<&MeshMaterial3d<GizmoMaterial>>,
    mut materials: ResMut<Assets<GizmoMaterial>>,
) {
    let handle_entity = trigger.target();

    // Highlight all child meshes (arrow shaft, cone, and axis line)
    if let Ok(children) = handle_query.get(handle_entity) {
        for child in children.iter() {
            if let Ok(material_handle) = material_query.get(child) {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    // Increase emissive on hover for highlight effect
                    material.emissive_r = 0.5;
                    material.emissive_g = 0.5;
                    material.emissive_b = 0.5;
                }
            }
        }
    }
}

/// Remove highlight from gizmo handle when hover ends (entity observer)
pub fn on_gizmo_hover_end(
    trigger: Trigger<Pointer<Out>>,
    handle_query: Query<&Children, With<GizmoHandle>>,
    material_query: Query<&MeshMaterial3d<GizmoMaterial>>,
    mut materials: ResMut<Assets<GizmoMaterial>>,
) {
    let handle_entity = trigger.target();

    // Restore original colors for all child meshes
    if let Ok(children) = handle_query.get(handle_entity) {
        for child in children.iter() {
            if let Ok(material_handle) = material_query.get(child) {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.emissive_r = 0.0;
                    material.emissive_g = 0.0;
                    material.emissive_b = 0.0;
                }
            }
        }
    }
}
