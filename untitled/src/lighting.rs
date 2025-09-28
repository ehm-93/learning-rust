use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::ShapeType, prelude::*};
use std::f32::consts::PI;

use crate::components::{LightSource, ShadowMesh, LightMask, Projectile};

/// Main shadow casting system that creates dynamic shadows from light sources
pub fn cast_shadows(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    light_query: Query<(&Transform, &LightSource)>,
    caster_query: Query<(&Transform, &Collider), (Without<LightSource>, With<RigidBody>, Without<Projectile>)>,
    shadow_query: Query<Entity, With<ShadowMesh>>,
    mask_query: Query<Entity, With<LightMask>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    // Clear all existing shadow meshes and light masks from previous frame
    for entity in shadow_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in mask_query.iter() {
        commands.entity(entity).despawn();
    }

    // Process each light source
    for (light_transform, light) in light_query.iter() {
        let light_pos = light_transform.translation.truncate();

        // Calculate light direction toward cursor position
        let light_direction = get_cursor_direction(light_pos, &windows, &camera_q);

        // Create full viewport darkness with light cone
        let darkness_mesh = create_viewport_with_light_cone(
            light_pos,
            light_direction,
            light.cone_angle,
            light.range,
        );

        commands.spawn((
            Mesh2d(meshes.add(darkness_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(
                Color::linear_rgba(0.0, 0.0, 0.0, 1.0) // Completely opaque darkness
            ))),
            Transform::from_xyz(0.0, 0.0, 10.0), // High Z to render above other objects
            LightMask,
        ));

        // Cast shadows for objects that have any part in the light cone
        for (caster_transform, collider) in caster_query.iter() {
            // Get the world-space vertices of this collider
            let vertices = get_collider_vertices(collider, caster_transform);

            if vertices.is_empty() {
                continue;
            }

            // Check if any vertex is within the light cone
            let mut any_vertex_in_cone = false;
            for vertex in &vertices {
                let to_vertex = *vertex - light_pos;
                let distance = to_vertex.length();

                if distance <= light.range {
                    let to_vertex_normalized = to_vertex.normalize_or_zero();
                    let angle_to_vertex = light_direction.angle_to(to_vertex_normalized).abs();

                    if angle_to_vertex <= light.cone_angle / 2.0 {
                        any_vertex_in_cone = true;
                        break;
                    }
                }
            }

            // If any vertex is in the cone, cast shadows for this object
            if any_vertex_in_cone {
                // Calculate shadow geometry for all edges facing away from light
                let shadow_vertices = cast_directional_shadow_from_vertices(
                    light_pos, &vertices, light.range
                );

                // Only create mesh if we have enough vertices for triangles
                if shadow_vertices.len() >= 3 {
                    let mesh = create_shadow_mesh(shadow_vertices);
                    let mesh_handle = meshes.add(mesh);

                    // Spawn shadow mesh entity
                    commands.spawn((
                        Mesh2d(mesh_handle),
                        MeshMaterial2d(materials.add(ColorMaterial::from_color(
                            Color::linear_rgba(0.0, 0.0, 0.0, 1.0) // Completely opaque shadows
                        ))),
                        Transform::from_xyz(0.0, 0.0, 11.0), // Above light mask
                        ShadowMesh,
                    ));
                }
            }
        }
    }
}

/// Gets the direction from light source toward cursor position
fn get_cursor_direction(
    light_pos: Vec2,
    windows: &Query<&Window>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Vec2 {
    // Default direction if cursor can't be found
    let mut light_direction = Vec2::Y;

    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), camera_q.single()) {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                light_direction = (world_pos - light_pos).normalize_or_zero();
            }
        }
    }

    light_direction
}

/// Creates a viewport darkness mesh with a cone-shaped hole cut out for light
fn create_viewport_with_light_cone(
    light_pos: Vec2,
    light_direction: Vec2,
    cone_angle: f32,
    _range: f32,
) -> Mesh {
    let half_cone_angle = cone_angle / 2.0;
    let darkness_angle = PI - half_cone_angle; // Darkness area on each side
    let triangle_radius = 3000.0; // Large enough to cover viewport

    let base_angle = light_direction.to_angle();

    // Calculate the cone edges
    let left_cone_edge = base_angle - half_cone_angle;
    let right_cone_edge = base_angle + half_cone_angle;

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Left darkness triangle (from left cone edge, going counter-clockwise)
    let left_start_angle = left_cone_edge;
    let left_end_angle = left_start_angle - darkness_angle;

    positions.push([light_pos.x, light_pos.y, 0.0]); // 0: light center
    positions.push([
        light_pos.x + left_start_angle.cos() * triangle_radius,
        light_pos.y + left_start_angle.sin() * triangle_radius,
        0.0
    ]); // 1: left cone edge
    positions.push([
        light_pos.x + left_end_angle.cos() * triangle_radius,
        light_pos.y + left_end_angle.sin() * triangle_radius,
        0.0
    ]); // 2: left darkness end

    indices.extend_from_slice(&[0, 1, 2]);

    // Right darkness triangle (from right cone edge, going clockwise)
    let right_start_angle = right_cone_edge;
    let right_end_angle = right_start_angle + darkness_angle;

    positions.push([
        light_pos.x + right_start_angle.cos() * triangle_radius,
        light_pos.y + right_start_angle.sin() * triangle_radius,
        0.0
    ]); // 3: right cone edge
    positions.push([
        light_pos.x + right_end_angle.cos() * triangle_radius,
        light_pos.y + right_end_angle.sin() * triangle_radius,
        0.0
    ]); // 4: right darkness end

    indices.extend_from_slice(&[0, 3, 4]);

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}

/// Enhanced shadow calculation that properly handles self-shadowing
fn cast_directional_shadow_from_vertices(
    light_pos: Vec2,
    vertices: &[Vec2],
    range: f32
) -> Vec<Vec2> {
    if vertices.len() < 3 {
        return vec![];
    }

    let mut shadow_vertices = Vec::new();
    let shadow_inset = 5.0; // Move shadow start point inward by this amount

    // Process each edge of the polygon
    for i in 0..vertices.len() {
        let v1 = vertices[i];
        let v2 = vertices[(i + 1) % vertices.len()];

        // Calculate edge vector and normal
        let edge_vec = v2 - v1;
        let edge_normal = Vec2::new(-edge_vec.y, edge_vec.x).normalize(); // Perpendicular to edge, pointing outward

        // Vector from edge midpoint to light
        let edge_midpoint = (v1 + v2) * 0.5;
        let to_light = (light_pos - edge_midpoint).normalize_or_zero();

        // Check if this edge faces away from the light (dot product < 0)
        let faces_away = edge_normal.dot(to_light) < 0.0;

        if faces_away {
            // This edge is a "silhouette edge" - it faces away from light
            // Calculate directions from light to each vertex of the edge
            let dir1 = (v1 - light_pos).normalize_or_zero();
            let dir2 = (v2 - light_pos).normalize_or_zero();

            // Move the shadow start points slightly inward along the light direction
            let shadow_start1 = v1 + dir1 * shadow_inset;
            let shadow_start2 = v2 + dir2 * shadow_inset;

            // Project vertices far away to create shadow
            let far1 = v1 + dir1 * range;
            let far2 = v2 + dir2 * range;

            // Create shadow quad: inset edge + far edge
            // Ensure proper winding order for consistent rendering
            shadow_vertices.extend_from_slice(&[shadow_start1, shadow_start2, far2, far1]);
        }
    }

    shadow_vertices
}

/// Extracts world-space vertices from a Rapier collider
/// Supports both cuboid and ball (circle) shapes
fn get_collider_vertices(collider: &Collider, transform: &Transform) -> Vec<Vec2> {
    match collider.raw.shape_type() {
        ShapeType::Cuboid => {
            // Get the cuboid shape and its dimensions
            let cuboid = collider.raw.as_cuboid().unwrap();
            let half_extents = cuboid.half_extents;

            // Define vertices in local space (counter-clockwise from bottom-left)
            let local_vertices = vec![
                Vec2::new(-half_extents.x, -half_extents.y), // Bottom-left
                Vec2::new(half_extents.x, -half_extents.y),  // Bottom-right
                Vec2::new(half_extents.x, half_extents.y),   // Top-right
                Vec2::new(-half_extents.x, half_extents.y),  // Top-left
            ];

            // Transform vertices from local space to world space
            local_vertices.iter().map(|&v| {
                // Apply rotation
                let rotated = transform.rotation * Vec3::new(v.x, v.y, 0.0);
                // Apply translation
                Vec2::new(rotated.x, rotated.y) + transform.translation.truncate()
            }).collect()
        }
        ShapeType::Ball => {
            // For circles, create an approximation using a polygon
            let ball = collider.raw.as_ball().unwrap();
            let radius = ball.radius;
            let segments = 16; // 16 sides for smooth circle approximation

            let mut vertices = Vec::new();
            for i in 0..segments {
                let angle = (i as f32 / segments as f32) * 2.0 * PI;
                let local_vertex = Vec2::new(
                    angle.cos() * radius,
                    angle.sin() * radius
                );

                // Transform from local space to world space
                let rotated = transform.rotation * Vec3::new(local_vertex.x, local_vertex.y, 0.0);
                let world_vertex = Vec2::new(rotated.x, rotated.y) + transform.translation.truncate();
                vertices.push(world_vertex);
            }
            vertices
        }
        _ => {
            // TODO: Add support for other shape types (polygons, etc.)
            vec![]
        }
    }
}

/// Creates a Bevy mesh from shadow geometry vertices
/// Converts quads to triangles for GPU rendering
fn create_shadow_mesh(vertices: Vec<Vec2>) -> Mesh {
    // Create mesh with triangle list topology for GPU rendering
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Convert each quad (4 vertices) into two triangles (6 vertices)
    for quad in vertices.chunks(4) {
        if quad.len() == 4 {
            let base_idx = positions.len() as u32;

            // Add all 4 vertices of the quad to positions
            for vertex in quad {
                positions.push([vertex.x, vertex.y, 0.0]);
            }

            // Create two triangles from the quad
            // Triangle 1: vertices 0, 1, 2
            // Triangle 2: vertices 0, 2, 3
            indices.extend_from_slice(&[
                base_idx,     base_idx + 1, base_idx + 2, // First triangle
                base_idx,     base_idx + 2, base_idx + 3, // Second triangle
            ]);
        }
    }

    // Set mesh attributes required by Bevy's renderer
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
}
