use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    constants::*,
    world::scenes::manager::{Scene, SceneEntity, SceneId},
};
use super::{PlayerBundle, Player};

/// Service for spawning player entities with proper scene tracking
pub struct PlayerSpawner;

impl PlayerSpawner {
    /// Spawn a player entity in a specific scene at the given position
    pub fn spawn_in_scene<T: Scene>(world: &mut World, position: Vec3) -> Entity {
        // Get mesh and material resources
        let mesh_handle = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Circle::new(PLAYER_RADIUS))
        };

        let material_handle = {
            let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
            materials.add(Color::WHITE)
        };

        // Spawn player with all necessary components
        world.spawn((
            // Visual components
            Mesh2d(mesh_handle),
            MeshMaterial2d(material_handle),
            Transform::from_translation(position),

            // Player bundle (core player components)
            PlayerBundle::default(),

            // Physics components
            RigidBody::Dynamic,
            Collider::ball(PLAYER_RADIUS),
            LockedAxes::ROTATION_LOCKED,
            Velocity::zero(),
            ActiveEvents::COLLISION_EVENTS,

            // Scene tracking
            SceneEntity {
                scene_id: SceneId::of::<T>(),
            },
        )).id()
    }

    /// Spawn a player entity without scene tracking (for testing or standalone use)
    pub fn spawn_standalone(world: &mut World, position: Vec3) -> Entity {
        // Get mesh and material resources
        let mesh_handle = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Circle::new(PLAYER_RADIUS))
        };

        let material_handle = {
            let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
            materials.add(Color::WHITE)
        };

        // Spawn player with all necessary components (no scene tracking)
        world.spawn((
            // Visual components
            Mesh2d(mesh_handle),
            MeshMaterial2d(material_handle),
            Transform::from_translation(position),

            // Player bundle (core player components)
            PlayerBundle::default(),

            // Physics components
            RigidBody::Dynamic,
            Collider::ball(PLAYER_RADIUS),
            LockedAxes::ROTATION_LOCKED,
            Velocity::zero(),
            ActiveEvents::COLLISION_EVENTS,
        )).id()
    }

    /// Find the player entity in the world (if it exists)
    pub fn find_player(world: &World) -> Option<Entity> {
        // Find entities with Player component
        world.iter_entities()
            .find(|entity_ref| entity_ref.contains::<Player>())
            .map(|entity_ref| entity_ref.id())
    }

    /// Check if a player entity exists in the world
    pub fn player_exists(world: &World) -> bool {
        Self::find_player(world).is_some()
    }

    /// Get the player's current position (if player exists)
    pub fn get_player_position(world: &World) -> Option<Vec3> {
        if let Some(player_entity) = Self::find_player(world) {
            world.get::<Transform>(player_entity)
                .map(|transform| transform.translation)
        } else {
            None
        }
    }
}
