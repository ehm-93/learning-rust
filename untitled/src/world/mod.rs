//! World setup and management
//!
//! This module is responsible for world initialization and configuration.

pub mod dungeon;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    constants::*,
    resources::*,
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

    // Spawn player using centralized function
    spawn_player(&mut commands, &mut meshes, &mut materials);

    // Generate procedural dungeon rooms
    generate_dungeon_rooms(&mut commands, &mut meshes, &mut materials, 1);
}

/// Initializes or resets the game state - used for both startup and restart
pub fn initialize_game_state(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut score: ResMut<Score>,
    mut game_state: ResMut<GameState>,
    mut fire_timer: ResMut<FireTimer>,
    mut player_query: Query<(&mut Health, &mut Transform, &mut Velocity, &mut Dash, &mut GrenadeThrower), With<Player>>,
) {
    // Reset game state
    *game_state = GameState::Playing;
    score.current = 0;

    // Reset player state completely
    if let Ok((mut health, mut transform, mut velocity, mut dash, mut grenade_thrower)) = player_query.single_mut() {
        // Reset health
        health.current = health.max;

        // Reset position to center
        transform.translation = Vec3::new(0.0, 0.0, 0.0);

        // Reset velocity
        velocity.linvel = Vec2::ZERO;
        velocity.angvel = 0.0;

        // Reset dash state
        *dash = Dash::new();

        // Reset grenade thrower state
        *grenade_thrower = GrenadeThrower::new();
    } else {
        // Player doesn't exist, spawn a new one
        spawn_player(&mut commands, &mut meshes, &mut materials);
    }

    // Reset timers
    fire_timer.timer.reset();

    // Generate a completely new dungeon with enemies
    // This gives players a fresh experience every restart
    generate_dungeon_rooms(&mut commands, &mut meshes, &mut materials, 1);
}

/// Spawns a new player entity
pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
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
        LockedAxes::ROTATION_LOCKED,
        Velocity::zero(),
        ActiveEvents::COLLISION_EVENTS,
    ));
}

/// Cleans up game entities (enemies, projectiles) for restart
pub fn cleanup_game_entities(
    commands: &mut Commands,
    entities_query: &Query<Entity, (Or<(With<Enemy>, With<Projectile>)>, Without<Player>, Without<MainCamera>)>,
) {
    for entity in entities_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Cleans up all dungeon entities (walls, floors) for complete regeneration
pub fn cleanup_dungeon_entities(
    commands: &mut Commands,
    dungeon_query: &Query<Entity, With<DungeonWall>>,
    floor_query: &Query<Entity, (With<Mesh2d>, Without<Player>, Without<MainCamera>, Without<DungeonWall>, Without<Enemy>)>,
) {
    // Remove all dungeon walls
    for entity in dungeon_query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove all floor tiles (this is a bit of a hack - we're removing all mesh entities that aren't player/camera/walls/enemies)
    // In a more complex game, we'd want to tag floor tiles explicitly
    for entity in floor_query.iter() {
        commands.entity(entity).despawn();
    }
}


