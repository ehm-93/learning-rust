use bevy::prelude::*;

use crate::editor::core::materials::GridMaterial;

/// Grid configuration resource
#[derive(Resource)]
pub struct GridConfig {
    pub visible: bool,
    pub snap_enabled: bool,
    pub spacing: f32, // Grid spacing in meters
    pub size: f32,    // Total grid size (half-extent from origin)
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            visible: true,
            snap_enabled: false,
            spacing: 0.5,
            size: 50.0,
        }
    }
}

/// Marker component for grid lines
#[derive(Component)]
pub struct GridEntity;

/// Setup the grid
pub fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid_materials: ResMut<Assets<GridMaterial>>,
    config: Res<GridConfig>,
) {
    if !config.visible {
        return;
    }

    // Create custom grid material with distance fade
    let grid_color = LinearRgba::new(0.3, 0.3, 0.3, 0.5);
    let material = grid_materials.add(GridMaterial {
        color: grid_color,
        fade_start: 10.0,  // Start fading at 10 meters
        fade_end: 50.0,    // Fully faded at 50 meters
    });

    let num_lines = (config.size / config.spacing) as i32;

    // Lines parallel to X-axis (skip the origin line, we'll draw it separately)
    for i in -num_lines..=num_lines {
        if i == 0 {
            continue; // Skip origin, will be drawn as green axis line
        }
        let z = i as f32 * config.spacing;
        let mesh = create_line_mesh(
            Vec3::new(-config.size, 0.01, z),
            Vec3::new(config.size, 0.01, z),
        );

        commands.spawn((
            GridEntity,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Transform::IDENTITY,
        ));
    }

    // Lines parallel to Z-axis (skip the origin line, we'll draw it separately)
    for i in -num_lines..=num_lines {
        if i == 0 {
            continue; // Skip origin, will be drawn as red axis line
        }
        let x = i as f32 * config.spacing;
        let mesh = create_line_mesh(
            Vec3::new(x, 0.01, -config.size),
            Vec3::new(x, 0.01, config.size),
        );

        commands.spawn((
            GridEntity,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Transform::IDENTITY,
        ));
    }

    // X-axis origin line (red) - runs along X direction
    let x_axis_material = grid_materials.add(GridMaterial {
        color: LinearRgba::new(1.0, 0.0, 0.0, 0.8), // Bright red
        fade_start: 10.0,
        fade_end: 50.0,
    });
    let x_axis_mesh = create_line_mesh(
        Vec3::new(-config.size, 0.02, 0.0), // Slightly higher to be visible above grid
        Vec3::new(config.size, 0.02, 0.0),
    );
    commands.spawn((
        GridEntity,
        Mesh3d(meshes.add(x_axis_mesh)),
        MeshMaterial3d(x_axis_material),
        Transform::IDENTITY,
    ));

    // Z-axis origin line (green) - runs along Z direction
    let z_axis_material = grid_materials.add(GridMaterial {
        color: LinearRgba::new(0.0, 1.0, 0.0, 0.8), // Bright green
        fade_start: 10.0,
        fade_end: 50.0,
    });
    let z_axis_mesh = create_line_mesh(
        Vec3::new(0.0, 0.02, -config.size), // Slightly higher to be visible above grid
        Vec3::new(0.0, 0.02, config.size),
    );
    commands.spawn((
        GridEntity,
        Mesh3d(meshes.add(z_axis_mesh)),
        MeshMaterial3d(z_axis_material),
        Transform::IDENTITY,
    ));
}

/// Create a line mesh between two points
fn create_line_mesh(start: Vec3, end: Vec3) -> Mesh {
    let positions = vec![
        [start.x, start.y, start.z],
        [end.x, end.y, end.z],
    ];

    let normals = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0]];
    let indices = vec![0, 1];

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::LineList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices.into()))
}

/// Toggle grid snapping with G key
pub fn toggle_snap(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<GridConfig>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        config.snap_enabled = !config.snap_enabled;
        info!("Grid snap: {}", if config.snap_enabled { "ON" } else { "OFF" });
    }
}

/// Snap a position to the grid
pub fn snap_to_grid(position: Vec3, spacing: f32) -> Vec3 {
    Vec3::new(
        (position.x / spacing).round() * spacing,
        (position.y / spacing).round() * spacing,
        (position.z / spacing).round() * spacing,
    )
}

/// Snap a rotation to 15° increments
pub fn snap_rotation(angle: f32) -> f32 {
    let increment = 15.0_f32.to_radians();
    (angle / increment).round() * increment
}
