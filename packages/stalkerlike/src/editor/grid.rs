use bevy::prelude::*;

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
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GridConfig>,
) {
    if !config.visible {
        return;
    }

    // Create grid lines
    let grid_color = Color::srgba(0.3, 0.3, 0.3, 0.5);
    let material = materials.add(StandardMaterial {
        base_color: grid_color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let num_lines = (config.size / config.spacing) as i32;

    // Lines parallel to X-axis
    for i in -num_lines..=num_lines {
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

    // Lines parallel to Z-axis
    for i in -num_lines..=num_lines {
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
