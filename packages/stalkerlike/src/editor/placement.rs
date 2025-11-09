use bevy::prelude::*;

use super::components::{EditorCamera, EditorEntity};
use super::grid::{snap_to_grid, GridConfig};
use super::primitives::{AssetCatalog, PrimitiveDefinition};

/// Resource tracking the current placement state
#[derive(Resource, Default)]
pub struct PlacementState {
    pub active: bool,
    pub preview_entity: Option<Entity>,
    pub selected_primitive: Option<PrimitiveDefinition>,
}

/// Marker component for preview entities
#[derive(Component)]
pub struct PreviewEntity;

/// Start placing a primitive
pub fn start_placement(
    placement_state: &mut PlacementState,
    primitive: PrimitiveDefinition,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Clean up any existing preview
    if let Some(entity) = placement_state.preview_entity {
        commands.entity(entity).despawn();
    }

    // Create preview entity
    let mesh = primitive.primitive_type.create_mesh(primitive.default_size);
    let mut material_color = primitive.color;
    material_color.set_alpha(0.5); // Semi-transparent

    let preview = commands
        .spawn((
            PreviewEntity,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: material_color,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ))
        .id();

    placement_state.preview_entity = Some(preview);
    placement_state.selected_primitive = Some(primitive);
    placement_state.active = true;
}

/// Update preview position to follow mouse cursor
pub fn update_preview_position(
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    windows: Query<&Window>,
    mut preview_query: Query<&mut Transform, With<PreviewEntity>>,
    placement_state: Res<PlacementState>,
    grid_config: Res<GridConfig>,
) {
    if !placement_state.active {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Cast ray from camera through cursor
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Intersect with ground plane (Y=0)
    if let Some(distance) = ray_plane_intersection(ray.origin, ray.direction.as_vec3(), Vec3::ZERO, Vec3::Y) {
        let mut point = ray.origin + ray.direction.as_vec3() * distance;

        // Apply grid snapping if enabled
        if grid_config.snap_enabled {
            point = snap_to_grid(point, grid_config.spacing);
        }

        // Update preview position
        for mut transform in preview_query.iter_mut() {
            transform.translation = point;
        }
    }
}

/// Place the object on mouse click
pub fn place_object(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut placement_state: ResMut<PlacementState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    preview_query: Query<&Transform, With<PreviewEntity>>,
    camera_query: Query<&EditorCamera>,
) {
    if !placement_state.active {
        return;
    }

    // Check if camera mouse is locked (if so, we shouldn't be placing)
    if let Ok(camera) = camera_query.single() {
        if camera.mouse_locked {
            return;
        }
    }

    // Cancel placement on ESC
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Some(entity) = placement_state.preview_entity {
            commands.entity(entity).despawn();
        }
        placement_state.active = false;
        placement_state.preview_entity = None;
        placement_state.selected_primitive = None;
        return;
    }

    // Place object on left click
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(primitive) = &placement_state.selected_primitive {
            if let Ok(preview_transform) = preview_query.single() {
                // Spawn the actual object
                let mesh = primitive.primitive_type.create_mesh(primitive.default_size);
                commands.spawn((
                    EditorEntity,
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: primitive.color,
                        ..default()
                    })),
                    *preview_transform,
                ));

                info!("Placed {} at {:?}", primitive.name, preview_transform.translation);
            }
        }

        // Continue placement mode for multiple placements
        // User needs to press ESC to exit
    }
}

/// Ray-plane intersection helper
fn ray_plane_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    plane_origin: Vec3,
    plane_normal: Vec3,
) -> Option<f32> {
    let denom = plane_normal.dot(ray_direction);
    if denom.abs() < 0.0001 {
        return None; // Ray is parallel to plane
    }

    let t = (plane_origin - ray_origin).dot(plane_normal) / denom;
    if t >= 0.0 {
        Some(t)
    } else {
        None
    }
}
