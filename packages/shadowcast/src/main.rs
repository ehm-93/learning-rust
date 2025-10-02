use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::ShapeType, prelude::*};
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Initialize Rapier physics with 100 pixels per meter scaling
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // Enable debug rendering to visualize colliders
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        // Run animation before shadow casting so transforms update first
        .add_systems(Update, (animate_light, animate_objects, cast_shadows).chain())
        .run();
}

/// Component marking an entity as a light source
#[derive(Component)]
struct LightSource {
    /// Maximum distance shadows can be cast
    range: f32,
    /// Orbital radius around center
    orbit_radius: f32,
    /// Speed of orbital motion (radians per second)
    orbit_speed: f32,
    /// Light cone angle in radians (90 degrees = π/2)
    cone_angle: f32,
}

/// Component marking an entity as something that casts shadows
#[derive(Component)]
struct ShadowCaster {
    /// Distance from center to orbit around
    orbit_radius: f32,
    /// Speed of orbital motion (radians per second)
    orbit_speed: f32,
    /// Starting angle offset for orbital position (radians)
    orbit_phase: f32,
    /// Speed of rotation in place (radians per second)
    spin_speed: f32,
}

/// Component marking shadow mesh entities for cleanup
#[derive(Component)]
struct ShadowMesh;

/// Component marking the darkness/light mask mesh
#[derive(Component)]
struct LightMask;

/// Initial scene setup - creates light source and shadow casting objects
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create 2D camera
    commands.spawn(Camera2d::default());

    // Light source orbiting inside the shapes
    let mut light_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    let light_segments = 16;
    let light_radius = 10.0; // Half of the original 20x20 size
    let mut light_positions = Vec::new();
    let mut light_indices = Vec::new();

    // Center vertex
    light_positions.push([0.0, 0.0, 0.0]);

    // Circle perimeter vertices
    for i in 0..=light_segments {
        let angle = (i as f32 / light_segments as f32) * 2.0 * PI;
        light_positions.push([
            angle.cos() * light_radius,
            angle.sin() * light_radius,
            0.0
        ]);
    }

    // Create triangles
    for i in 0..light_segments {
        light_indices.extend_from_slice(&[0, i + 1, i + 2]);
    }

    light_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, light_positions);
    light_mesh.insert_indices(bevy::render::mesh::Indices::U32(light_indices));

    commands.spawn((
        Mesh2d(meshes.add(light_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::linear_rgb(1.0, 1.0, 0.0) // Bright yellow
        ))),
        Transform::from_xyz(70.0, 0.0, 2.0), // Start at right side of smaller inner orbit
        LightSource {
            range: 600.0,       // Shadow distance
            orbit_radius: 70.0,  // Smaller radius (was 100, now 70)
            orbit_speed: -0.5,   // Counter-clockwise (opposite to shapes)
            cone_angle: PI / 2.0, // 90 degree cone
        },
    ));

    // Blue square - starts at 0 degrees (right side)
    let mut blue_square_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    let square_size = 100.0;
    let half_size = square_size / 2.0;
    let square_positions = vec![
        [-half_size, -half_size, 0.0], // Bottom-left
        [half_size, -half_size, 0.0],  // Bottom-right
        [half_size, half_size, 0.0],   // Top-right
        [-half_size, half_size, 0.0],  // Top-left
    ];
    let square_indices = vec![0, 1, 2, 0, 2, 3];

    blue_square_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, square_positions);
    blue_square_mesh.insert_indices(bevy::render::mesh::Indices::U32(square_indices));

    commands.spawn((
        Mesh2d(meshes.add(blue_square_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::linear_rgb(0.0, 0.0, 1.0) // Blue
        ))),
        Transform::from_xyz(160.0, 0.0, -0.1), // Below darkness layer
        RigidBody::Fixed,
        Collider::cuboid(50.0, 50.0),
        ShadowCaster {
            orbit_radius: 160.0,
            orbit_speed: 0.3,    // Slower orbital speed
            orbit_phase: 0.0,    // 0 degrees starting position
            spin_speed: 0.5,     // Slower spin
        },
    ));

    // Red square - starts at 120 degrees
    let mut red_square_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    let red_size = 80.0;
    let red_half = red_size / 2.0;
    let red_positions = vec![
        [-red_half, -red_half, 0.0], // Bottom-left
        [red_half, -red_half, 0.0],  // Bottom-right
        [red_half, red_half, 0.0],   // Top-right
        [-red_half, red_half, 0.0],  // Top-left
    ];
    let red_indices = vec![0, 1, 2, 0, 2, 3];

    red_square_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, red_positions);
    red_square_mesh.insert_indices(bevy::render::mesh::Indices::U32(red_indices));

    commands.spawn((
        Mesh2d(meshes.add(red_square_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::linear_rgb(1.0, 0.0, 0.0) // Red
        ))),
        Transform::from_xyz(
            120.0_f32.to_radians().cos() * 140.0,
            120.0_f32.to_radians().sin() * 140.0,
            -0.1
        ), // Below darkness layer
        RigidBody::Fixed,
        Collider::cuboid(40.0, 40.0),
        ShadowCaster {
            orbit_radius: 140.0,
            orbit_speed: 0.3,    // Same orbital speed
            orbit_phase: 120.0_f32.to_radians(), // 120 degrees (2π/3)
            spin_speed: 1.0,     // Slower spin
        },
    ));

    // Green rectangle - starts at 240 degrees
    let mut green_rect_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    let rect_width = 60.0;
    let rect_height = 120.0;
    let rect_half_w = rect_width / 2.0;
    let rect_half_h = rect_height / 2.0;
    let rect_positions = vec![
        [-rect_half_w, -rect_half_h, 0.0], // Bottom-left
        [rect_half_w, -rect_half_h, 0.0],  // Bottom-right
        [rect_half_w, rect_half_h, 0.0],   // Top-right
        [-rect_half_w, rect_half_h, 0.0],  // Top-left
    ];
    let rect_indices = vec![0, 1, 2, 0, 2, 3];

    green_rect_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, rect_positions);
    green_rect_mesh.insert_indices(bevy::render::mesh::Indices::U32(rect_indices));

    commands.spawn((
        Mesh2d(meshes.add(green_rect_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::linear_rgb(0.0, 1.0, 0.0) // Green
        ))),
        Transform::from_xyz(
            240.0_f32.to_radians().cos() * 200.0,
            240.0_f32.to_radians().sin() * 200.0,
            -0.1
        ), // Below darkness layer
        RigidBody::Fixed,
        Collider::cuboid(30.0, 60.0),
        ShadowCaster {
            orbit_radius: 200.0,
            orbit_speed: 0.3,    // Same orbital speed
            orbit_phase: 240.0_f32.to_radians(), // 240 degrees (4π/3)
            spin_speed: -0.75,   // Slower reverse spin
        },
    ));

    // Small circles orbiting outside the shapes in reverse direction
    let circle_count = 8;
    let circle_radius = 280.0; // Outside the largest shape orbit
    let circle_size = 20.0;

    for i in 0..circle_count {
        let angle = (i as f32 / circle_count as f32) * 2.0 * PI;
        let x = angle.cos() * circle_radius;
        let y = angle.sin() * circle_radius;

        // Create circular mesh
        let mut circle_mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );

        let segments = 16;
        let radius = circle_size / 2.0;
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        // Center vertex
        positions.push([0.0, 0.0, 0.0]);

        // Circle perimeter vertices
        for j in 0..=segments {
            let circle_angle = (j as f32 / segments as f32) * 2.0 * PI;
            positions.push([
                circle_angle.cos() * radius,
                circle_angle.sin() * radius,
                0.0
            ]);
        }

        // Create triangles
        for j in 0..segments {
            indices.extend_from_slice(&[0, j + 1, j + 2]);
        }

        circle_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        circle_mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

        commands.spawn((
            Mesh2d(meshes.add(circle_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(
                Color::linear_rgb(1.0, 1.0, 1.0) // White circles
            ))),
            Transform::from_xyz(x, y, -0.1), // Below darkness layer
            RigidBody::Fixed,
            Collider::ball(circle_size / 2.0), // Use ball collider for proper circle physics
            ShadowCaster {
                orbit_radius: circle_radius,
                orbit_speed: -0.3,   // Reverse direction from main shapes
                orbit_phase: angle,  // Starting position
                spin_speed: 0.0,     // No spinning
            },
        ));
    }
}

/// Animates the light source - makes it orbit counter-clockwise inside the shapes
fn animate_light(
    time: Res<Time>,
    mut light_query: Query<(&mut Transform, &LightSource)>,
) {
    for (mut transform, light) in light_query.iter_mut() {
        let elapsed = time.elapsed_secs();

        // Orbit around the center point (0,0)
        let orbit_angle = elapsed * light.orbit_speed;
        let x = orbit_angle.cos() * light.orbit_radius;
        let y = orbit_angle.sin() * light.orbit_radius;

        transform.translation.x = x;
        transform.translation.y = y;
        // Keep Z at 2 for proper layering above everything else
        transform.translation.z = 2.0;
    }
}

/// Animates shadow casting objects - makes them orbit around center and spin
fn animate_objects(
    time: Res<Time>,
    mut caster_query: Query<(&mut Transform, &ShadowCaster)>,
) {
    for (mut transform, caster) in caster_query.iter_mut() {
        let elapsed = time.elapsed_secs();

        // Calculate orbital position around the center (light source at 0,0)
        // Add the phase offset to create starting position variety
        let orbit_angle = elapsed * caster.orbit_speed + caster.orbit_phase;
        let x = orbit_angle.cos() * caster.orbit_radius;
        let y = orbit_angle.sin() * caster.orbit_radius;

        // Calculate rotation in place
        let spin_angle = elapsed * caster.spin_speed;

        // Update transform
        transform.translation.x = x;
        transform.translation.y = y;
        transform.rotation = Quat::from_rotation_z(spin_angle);
    }
}

/// Creates a viewport darkness mesh with a cone-shaped hole cut out for light
fn create_viewport_with_light_cone(
    light_pos: Vec2,
    light_direction: Vec2,
    cone_angle: f32,
    range: f32,
) -> Mesh {
    let half_cone_angle = cone_angle / 2.0; // 45 degrees for 90° cone
    let darkness_angle = PI - half_cone_angle; // 135 degrees for each darkness triangle
    let triangle_radius = 3000.0; // Much larger to cover entire viewport

    let base_angle = light_direction.to_angle();

    // Calculate the cone edges
    let left_cone_edge = base_angle - half_cone_angle;
    let right_cone_edge = base_angle + half_cone_angle;

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Left darkness triangle (135° starting from left cone edge, going counter-clockwise)
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

    // Right darkness triangle (135° starting from right cone edge, going clockwise)
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

/// Main shadow casting system with proper light masking
fn cast_shadows(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    light_query: Query<(&Transform, &LightSource)>,
    caster_query: Query<(&Transform, &Collider), With<ShadowCaster>>,
    shadow_query: Query<Entity, With<ShadowMesh>>,
    mask_query: Query<Entity, With<LightMask>>,
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

        // Point light toward center of orbits (0, 0)
        let light_direction = (Vec2::ZERO - light_pos).normalize_or_zero();

        // Create full viewport darkness
        let darkness_mesh = create_viewport_with_light_cone(
            light_pos,
            light_direction,
            light.cone_angle,
            light.range,
        );

        commands.spawn((
            Mesh2d(meshes.add(darkness_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(
                Color::linear_rgba(0.0, 0.0, 0.0, 1.0) // Completely opaque black
            ))),
            Transform::from_xyz(0.0, 0.0, 0.5), // Above background shapes
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
                            Color::linear_rgba(0.0, 0.0, 0.0, 1.0) // Completely opaque black like darkness
                        ))),
                        Transform::from_xyz(0.0, 0.0, 1.0), // Above light cone
                        ShadowMesh,
                    ));
                }
            }
        }
    }
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
