use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    constants::*,
};

/// Disables gravity for the 2D physics world
pub fn disable_gravity(mut query: Query<&mut RapierConfiguration>) {
    for mut config in &mut query {
        config.gravity = Vec2::ZERO;
    }
}

/// Sets up the initial game world with player, obstacles, and boundaries
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn 2D camera with following component
    commands.spawn((
        Camera2d,
        MainCamera,
    ));

    // Spawn player as a white circle
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Player,
        Team::Player,
        Health::new(PLAYER_MAX_HEALTH),
        Dash::new(),
        GrenadeThrower::new(),
        RigidBody::Dynamic,
        Collider::ball(PLAYER_RADIUS),
        // Lock rotation so the player doesn't spin
        LockedAxes::ROTATION_LOCKED,
        // Add Velocity component for movement
        Velocity::zero(),
        // Enable collision events for damage detection
        ActiveEvents::COLLISION_EVENTS,
    ));

    // Spawn obstacles as gray rectangles with varied rotations
    let obstacle_data = [
        (Vec2::new(150.0, 100.0), 0.3),
        (Vec2::new(-150.0, -100.0), -0.7),
        (Vec2::new(200.0, -150.0), 0.5),
        (Vec2::new(-200.0, 150.0), -0.2),
        (Vec2::new(0.0, 200.0), 0.8),
        (Vec2::new(0.0, -200.0), -0.4),
    ];

    for (pos, rotation) in obstacle_data {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(OBSTACLE_WIDTH, OBSTACLE_HEIGHT))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))), // Gray obstacles
            Transform::from_translation(pos.extend(0.0)).with_rotation(Quat::from_rotation_z(rotation)),
            Obstacle,
            RigidBody::Fixed,
            Collider::cuboid(OBSTACLE_WIDTH / 2.0, OBSTACLE_HEIGHT / 2.0),
        ));
    }

    // Spawn boundary walls (invisible but solid)
    spawn_boundaries(&mut commands, &mut meshes, &mut materials);
}

/// Spawns the invisible boundary walls around the map
fn spawn_boundaries(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let half_width = MAP_WIDTH / 2.0;
    let half_height = MAP_HEIGHT / 2.0;
    let half_thickness = WALL_THICKNESS / 2.0;

    // Top wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(MAP_WIDTH + WALL_THICKNESS, WALL_THICKNESS))),
        MeshMaterial2d(materials.add(Color::srgba(0.2, 0.2, 0.2, 0.3))), // Semi-transparent dark gray
        Transform::from_translation(Vec3::new(0.0, half_height + half_thickness, -0.1)),
        Boundary,
        RigidBody::Fixed,
        Collider::cuboid((MAP_WIDTH + WALL_THICKNESS) / 2.0, WALL_THICKNESS / 2.0),
    ));

    // Bottom wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(MAP_WIDTH + WALL_THICKNESS, WALL_THICKNESS))),
        MeshMaterial2d(materials.add(Color::srgba(0.2, 0.2, 0.2, 0.3))),
        Transform::from_translation(Vec3::new(0.0, -half_height - half_thickness, -0.1)),
        Boundary,
        RigidBody::Fixed,
        Collider::cuboid((MAP_WIDTH + WALL_THICKNESS) / 2.0, WALL_THICKNESS / 2.0),
    ));

    // Left wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, MAP_HEIGHT))),
        MeshMaterial2d(materials.add(Color::srgba(0.2, 0.2, 0.2, 0.3))),
        Transform::from_translation(Vec3::new(-half_width - half_thickness, 0.0, -0.1)),
        Boundary,
        RigidBody::Fixed,
        Collider::cuboid(WALL_THICKNESS / 2.0, MAP_HEIGHT / 2.0),
    ));

    // Right wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, MAP_HEIGHT))),
        MeshMaterial2d(materials.add(Color::srgba(0.2, 0.2, 0.2, 0.3))),
        Transform::from_translation(Vec3::new(half_width + half_thickness, 0.0, -0.1)),
        Boundary,
        RigidBody::Fixed,
        Collider::cuboid(WALL_THICKNESS / 2.0, MAP_HEIGHT / 2.0),
    ));
}
