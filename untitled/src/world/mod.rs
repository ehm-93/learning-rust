//! World setup and management
//!
//! This module is responsible for world initialization and configuration.

pub mod dungeon;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    constants::*,
};
use dungeon::generate_dungeon_rooms;

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
        LightSource::new(600.0, std::f32::consts::PI * 0.5), // 600 units range, 90 degree cone
        RigidBody::Dynamic,
        Collider::ball(PLAYER_RADIUS),
        // Lock rotation so the player doesn't spin
        LockedAxes::ROTATION_LOCKED,
        // Add Velocity component for movement
        Velocity::zero(),
        // Enable collision events for damage detection
        ActiveEvents::COLLISION_EVENTS,
    ));

    // Generate procedural dungeon rooms
    generate_dungeon_rooms(&mut commands, &mut meshes, &mut materials, 1);
}
