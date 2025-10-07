//! Behavior stress test scene
//!
//! Spawns multiple entities with different behavior combinations to test
//! the behavior system and Lua integration.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::behavior::{BehaviorComponent, BehaviorRegistry, Params, ParamValue};

// Simple marker component for test entities
#[derive(Component)]
struct TestEntity;

/// Plugin that sets up the behavior stress test scene
pub struct BehaviorTestScenePlugin;

impl Plugin for BehaviorTestScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_test_scene.after(crate::packages::setup_package_system));
    }
}

/// System that spawns test entities with various behaviors
fn setup_test_scene(
    mut commands: Commands,
    behavior_registry: Res<BehaviorRegistry>,
) {
    info!("🎬 Setting up behavior test scene...");

    // Test 1: Entity with log_lifecycle behavior (Rust)
    {
        let mut params = Params::new();
        params.insert("name", ParamValue::String("TestEntity1".to_string()));
        
        let entity = commands.spawn((
            TestEntity,
            Transform::from_xyz(-200.0, 0.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::ball(20.0),
        )).id();
        
        params.insert("entity", ParamValue::EntityId(entity));
        
        if let Some(behavior) = behavior_registry.instantiate("log_lifecycle", params) {
            commands.entity(entity).insert(BehaviorComponent::new(vec![behavior]));
            info!("✅ Spawned entity with log_lifecycle behavior");
        }
    }

    // Test 2: Entity with lifetime behavior (Rust) - will despawn after 5 seconds
    {
        let mut params = Params::new();
        params.insert("lifetime", ParamValue::Float(5.0));
        
        let entity = commands.spawn((
            TestEntity,
            Transform::from_xyz(-100.0, 0.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::ball(15.0),
        )).id();
        
        params.insert("entity", ParamValue::EntityId(entity));
        
        if let Some(behavior) = behavior_registry.instantiate("lifetime", params) {
            commands.entity(entity).insert(BehaviorComponent::new(vec![behavior]));
            info!("✅ Spawned entity with lifetime behavior (5s)");
        }
    }

    // Test 3: Entity with oscillate behavior (Rust)
    {
        let mut params = Params::new();
        params.insert("amplitude", ParamValue::Float(2.0));
        params.insert("frequency", ParamValue::Float(1.5));
        params.insert("axis", ParamValue::Vec3(Vec3::Y));
        
        let entity = commands.spawn((
            TestEntity,
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::ball(18.0),
        )).id();
        
        params.insert("entity", ParamValue::EntityId(entity));
        
        if let Some(behavior) = behavior_registry.instantiate("oscillate", params) {
            commands.entity(entity).insert(BehaviorComponent::new(vec![behavior]));
            info!("✅ Spawned entity with oscillate behavior");
        }
    }

    // Test 4: Entity with spinner behavior (Lua)
    {
        let mut params = Params::new();
        
        let entity = commands.spawn((
            TestEntity,
            Transform::from_xyz(100.0, 0.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::ball(22.0),
        )).id();
        
        params.insert("entity", ParamValue::EntityId(entity));
        params.insert("rotation_speed", ParamValue::Float(3.0));
        
        if let Some(behavior) = behavior_registry.instantiate("spinner", params) {
            commands.entity(entity).insert(BehaviorComponent::new(vec![behavior]));
            info!("✅ Spawned entity with spinner behavior (Lua)");
        } else {
            warn!("⚠️  Failed to create spinner behavior - check if test package loaded");
        }
    }

    // Test 5: Entity with pulse_and_die behavior (Lua)
    {
        let mut params = Params::new();
        
        let entity = commands.spawn((
            TestEntity,
            Transform::from_xyz(200.0, 0.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::ball(25.0),
        )).id();
        
        params.insert("entity", ParamValue::EntityId(entity));
        params.insert("lifetime", ParamValue::Float(8.0));
        
        if let Some(behavior) = behavior_registry.instantiate("pulse_and_die", params) {
            commands.entity(entity).insert(BehaviorComponent::new(vec![behavior]));
            info!("✅ Spawned entity with pulse_and_die behavior (Lua)");
        } else {
            warn!("⚠️  Failed to create pulse_and_die behavior - check if test package loaded");
        }
    }

    // Test 6: Entity with seeker behavior (Lua)
    {
        let mut params = Params::new();
        
        let entity = commands.spawn((
            TestEntity,
            Transform::from_xyz(-200.0, 150.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::ball(16.0),
        )).id();
        
        params.insert("entity", ParamValue::EntityId(entity));
        params.insert("search_radius", ParamValue::Float(300.0));
        params.insert("seek_speed", ParamValue::Float(50.0));
        
        if let Some(behavior) = behavior_registry.instantiate("seeker", params) {
            commands.entity(entity).insert(BehaviorComponent::new(vec![behavior]));
            info!("✅ Spawned entity with seeker behavior (Lua)");
        } else {
            warn!("⚠️  Failed to create seeker behavior - check if test package loaded");
        }
    }

    // Test 7: Entity with multiple behaviors (Rust + Lua composite)
    {
        let mut params = Params::new();
        
        let entity = commands.spawn((
            TestEntity,
            Transform::from_xyz(0.0, -150.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::ball(20.0),
        )).id();
        
        params.insert("entity", ParamValue::EntityId(entity));
        
        let mut behaviors = Vec::new();
        
        // Add log_lifecycle
        let mut log_params = params.clone();
        log_params.insert("name", ParamValue::String("CompositeEntity".to_string()));
        if let Some(behavior) = behavior_registry.instantiate("log_lifecycle", log_params) {
            behaviors.push(behavior);
        }
        
        // Add collision_logger (Lua)
        if let Some(behavior) = behavior_registry.instantiate("collision_logger", params.clone()) {
            behaviors.push(behavior);
        }
        
        if !behaviors.is_empty() {
            commands.entity(entity).insert(BehaviorComponent::new(behaviors));
            info!("✅ Spawned entity with composite behaviors (log + collision_logger)");
        }
    }

    // Test 8: Entity with spinning_seeker (Lua composite behavior)
    {
        let mut params = Params::new();
        
        let entity = commands.spawn((
            TestEntity,
            Transform::from_xyz(200.0, 150.0, 0.0),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::ball(24.0),
        )).id();
        
        params.insert("entity", ParamValue::EntityId(entity));
        
        if let Some(behavior) = behavior_registry.instantiate("spinning_seeker", params) {
            commands.entity(entity).insert(BehaviorComponent::new(vec![behavior]));
            info!("✅ Spawned entity with spinning_seeker behavior (Lua composite)");
        } else {
            warn!("⚠️  Failed to create spinning_seeker behavior - check if test package loaded");
        }
    }

    info!("🎬 Behavior test scene setup complete!");
}
