use bevy::prelude::*;
use bevy::picking::events::{Pointer, DragStart, Drag, Over, Out};

use crate::editor::objects::selection::{SelectedEntity, Selected};
use crate::editor::viewport::grid::GridConfig;

// GIZMO LIFECYCLE:
// - spawn_gizmo() triggers on OnAdd<Selected> - creates gizmo when entity selected
// - update_gizmo_position() runs every frame - syncs gizmo to selected object
// - despawn_gizmo() triggers on OnRemove<Selected> - cleans up when deselected
// - Entity-specific observers (on_gizmo_*) handle all interaction events:
//   * on_gizmo_click: Start drag on Click
//   * on_gizmo_drag: Update transform during Drag
//   * on_gizmo_drag_end: End drag on DragEnd
//   * on_gizmo_hover/on_gizmo_hover_end: Visual feedback on Over/Out

/// Transform mode for the gizmo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

/// Resource tracking current transform mode
#[derive(Resource)]
pub struct GizmoState {
    pub mode: TransformMode,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            mode: TransformMode::Translate,
        }
    }
}

/// Axis identifier for gizmo handles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

/// Marker component for gizmo entities
#[derive(Component)]
pub struct GizmoHandle;

/// Marker component for the gizmo root entity
#[derive(Component)]
pub struct GizmoRoot;

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
    mut materials: ResMut<Assets<StandardMaterial>>,
    transform_query: Query<&Transform>,
) {
    let selected_entity = trigger.target();

    let Ok(transform) = transform_query.get(selected_entity) else {
        return;
    };

    let position = transform.translation;
    let handle_size = 0.1; // 10cm sphere handles
    let handle_offset = 1.5; // 1.5m from center

    // Create root entity for gizmo (at selected object's position)
    let root = commands.spawn((
        GizmoRoot,
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id();

    // Spawn handle meshes
    let sphere_mesh = meshes.add(Sphere::new(handle_size));

    // X-axis handle (red)
    let x_handle = commands.spawn((
        GizmoHandle,
        GizmoAxis::X,
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            emissive: LinearRgba::rgb(0.5, 0.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(handle_offset, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Pickable::default(), // Make it selectable
    ))
    .observe(on_gizmo_hover)
    .observe(on_gizmo_hover_end)
    .observe(on_gizmo_drag)
    .id();

    // Y-axis handle (blue)
    let y_handle = commands.spawn((
        GizmoHandle,
        GizmoAxis::Y,
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 1.0),
            emissive: LinearRgba::rgb(0.0, 0.0, 0.5),
            ..default()
        })),
        Transform::from_xyz(0.0, handle_offset, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Pickable::default(),
    ))
    .observe(on_gizmo_hover)
    .observe(on_gizmo_hover_end)
    .observe(on_gizmo_drag)
    .id();

    // Z-axis handle (green)
    let z_handle = commands.spawn((
        GizmoHandle,
        GizmoAxis::Z,
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            emissive: LinearRgba::rgb(0.0, 0.5, 0.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, handle_offset),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Pickable::default(),
    ))
    .observe(on_gizmo_hover)
    .observe(on_gizmo_hover_end)
    .observe(on_gizmo_drag)
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

/// Update gizmo position to follow selected object
pub fn update_gizmo_position(
    selected: Res<SelectedEntity>,
    selected_query: Query<&Transform, With<Selected>>,
    mut gizmo_query: Query<&mut Transform, (With<GizmoRoot>, Without<Selected>)>,
) {
    let Some(selected_entity) = selected.entity else {
        return;
    };

    let Ok(selected_transform) = selected_query.get(selected_entity) else {
        return;
    };

    for mut gizmo_transform in gizmo_query.iter_mut() {
        gizmo_transform.translation = selected_transform.translation;
    }
}

/// Handle drag on gizmo handle (entity observer) - fires continuously while dragging
fn on_gizmo_drag(
    drag: Trigger<Pointer<Drag>>,
    gizmo_state: Res<GizmoState>,
    handle_query: Query<&GizmoAxis, With<GizmoHandle>>,
    selected: Res<SelectedEntity>,
    mut selected_query: Query<&mut Transform, With<Selected>>,
    grid_config: Res<GridConfig>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    info!("Gizmo drag event: {:?}", drag.event());

    let handle_entity = drag.target();

    // Get the axis of the handle being dragged
    let Ok(axis) = handle_query.get(handle_entity) else {
        return;
    };

    // Get the selected object's transform
    let Some(selected_entity) = selected.entity else {
        return;
    };

    let Ok(mut transform) = selected_query.get_mut(selected_entity) else {
        return;
    };

    // Get drag delta from the event
    let delta = drag.event().delta;

    // Get camera for calculating drag scale
    let Ok((_camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Calculate drag scale based on distance from camera (farther = larger movements)
    let distance = (camera_transform.translation() - transform.translation).length();
    let drag_scale = distance * 0.002; // Tune this value for feel

    // Apply drag based on current mode and axis
    match gizmo_state.mode {
        TransformMode::Translate => {
            let mut movement = Vec3::ZERO;
            match axis {
                GizmoAxis::X => movement.x = delta.x * drag_scale,
                GizmoAxis::Y => movement.y = -delta.y * drag_scale, // Invert Y for intuitive up/down
                GizmoAxis::Z => movement.z = -delta.y * drag_scale, // Use vertical drag for Z
            }

            // Apply movement
            transform.translation += movement;

            // Apply grid snapping if enabled
            if grid_config.snap_enabled {
                let spacing = grid_config.spacing;
                match axis {
                    GizmoAxis::X => transform.translation.x = (transform.translation.x / spacing).round() * spacing,
                    GizmoAxis::Y => transform.translation.y = (transform.translation.y / spacing).round() * spacing,
                    GizmoAxis::Z => transform.translation.z = (transform.translation.z / spacing).round() * spacing,
                }
            }
        }
        TransformMode::Rotate => {
            // Rotation: use drag distance as angle delta
            let rotation_speed = 0.02; // Radians per pixel
            let angle_delta = delta.length() * rotation_speed * if delta.x + delta.y < 0.0 { -1.0 } else { 1.0 };

            let mut euler = transform.rotation.to_euler(EulerRot::XYZ);
            match axis {
                GizmoAxis::X => euler.0 += angle_delta,
                GizmoAxis::Y => euler.1 += angle_delta,
                GizmoAxis::Z => euler.2 += angle_delta,
            }

            // Apply rotation snapping if enabled (15 degrees)
            if grid_config.snap_enabled {
                let snap_angle = 15.0_f32.to_radians();
                match axis {
                    GizmoAxis::X => euler.0 = (euler.0 / snap_angle).round() * snap_angle,
                    GizmoAxis::Y => euler.1 = (euler.1 / snap_angle).round() * snap_angle,
                    GizmoAxis::Z => euler.2 = (euler.2 / snap_angle).round() * snap_angle,
                }
            }

            transform.rotation = Quat::from_euler(EulerRot::XYZ, euler.0, euler.1, euler.2);
        }
        TransformMode::Scale => {
            // Scale: vertical drag increases/decreases scale
            let scale_speed = 0.01;
            let scale_delta = -delta.y * scale_speed;

            match axis {
                GizmoAxis::X => transform.scale.x = (transform.scale.x + scale_delta).max(0.01),
                GizmoAxis::Y => transform.scale.y = (transform.scale.y + scale_delta).max(0.01),
                GizmoAxis::Z => transform.scale.z = (transform.scale.z + scale_delta).max(0.01),
            }
        }
    }
}

/// Highlight gizmo handle on hover (entity observer)
fn on_gizmo_hover(
    trigger: Trigger<Pointer<Over>>,
    handle_query: Query<&MeshMaterial3d<StandardMaterial>, With<GizmoHandle>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let handle_entity = trigger.target();

    if let Ok(material_handle) = handle_query.get(handle_entity) {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            // Increase emissive on hover for highlight effect
            material.emissive = LinearRgba::rgb(2.0, 2.0, 2.0);
        }
    }
}

/// Remove highlight from gizmo handle when hover ends (entity observer)
fn on_gizmo_hover_end(
    trigger: Trigger<Pointer<Out>>,
    handle_query: Query<(&GizmoAxis, &MeshMaterial3d<StandardMaterial>), With<GizmoHandle>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let handle_entity = trigger.target();

    if let Ok((axis, material_handle)) = handle_query.get(handle_entity) {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            // Restore original emissive based on axis color
            material.emissive = match axis {
                GizmoAxis::X => LinearRgba::rgb(0.5, 0.0, 0.0), // Red
                GizmoAxis::Y => LinearRgba::rgb(0.0, 0.0, 0.5), // Blue
                GizmoAxis::Z => LinearRgba::rgb(0.0, 0.5, 0.0), // Green
            };
        }
    }
}
