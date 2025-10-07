//! Lua World API Bridge
//!
//! Provides a Lua-friendly wrapper around Bevy's World for behaviors to interact with the ECS.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use mlua::prelude::*;
use std::cell::UnsafeCell;

/// A Lua-friendly wrapper around Bevy World
/// This gets passed to Lua behavior callbacks to allow them to interact with entities
pub struct LuaWorldApi {
    /// The entity that owns this behavior
    pub entity: Entity,

    /// Reference to the world (not directly accessible from Lua for safety)
    /// Using UnsafeCell to allow interior mutability
    world: UnsafeCell<*mut World>,
}

impl LuaWorldApi {
    /// Create a new LuaWorldApi
    ///
    /// # Safety
    /// The world pointer must remain valid for the lifetime of this object
    pub unsafe fn new(entity: Entity, world: *mut World) -> Self {
        Self {
            entity,
            world: UnsafeCell::new(world)
        }
    }

    /// Get a safe reference to the world
    ///
    /// # Safety
    /// Assumes the world pointer is still valid
    unsafe fn world(&self) -> &World {
        unsafe { &**self.world.get() }
    }

    /// Get a safe mutable reference to the world
    ///
    /// # Safety
    /// Assumes the world pointer is still valid
    unsafe fn world_mut(&self) -> &mut World {
        unsafe { &mut **self.world.get() }
    }
}

/// Convert LuaWorldApi to a Lua userdata that Lua can interact with
impl LuaUserData for LuaWorldApi {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Log a message (useful for debugging)
        methods.add_method("log", |_, _this, msg: String| {
            info!("[Lua Behavior] {}", msg);
            Ok(())
        });

        // Get entity position (if it has Transform)
        methods.add_method("get_position", |lua, this, ()| {
            unsafe {
                let world = this.world();
                if let Some(transform) = world.get::<Transform>(this.entity) {
                    let table = lua.create_table()?;
                    table.set("x", transform.translation.x)?;
                    table.set("y", transform.translation.y)?;
                    table.set("z", transform.translation.z)?;
                    Ok(table)
                } else {
                    lua.create_table()
                }
            }
        });

        // Set entity position (if it has Transform)
        methods.add_method("set_position", |_, this, (x, y, z): (f32, f32, f32)| {
            unsafe {
                let world = this.world_mut();
                if let Some(mut transform) = world.get_mut::<Transform>(this.entity) {
                    transform.translation.x = x;
                    transform.translation.y = y;
                    transform.translation.z = z;
                }
            }
            Ok(())
        });

        // Get entity rotation (if it has Transform)
        methods.add_method("get_rotation", |lua, this, ()| {
            unsafe {
                let world = this.world();
                if let Some(transform) = world.get::<Transform>(this.entity) {
                    // Return as euler angles for easier Lua manipulation
                    let (roll, pitch, yaw) = transform.rotation.to_euler(EulerRot::XYZ);
                    let table = lua.create_table()?;
                    table.set("roll", roll)?;
                    table.set("pitch", pitch)?;
                    table.set("yaw", yaw)?;
                    Ok(table)
                } else {
                    lua.create_table()
                }
            }
        });

        // Set entity rotation (if it has Transform)
        methods.add_method("set_rotation", |_, this, (roll, pitch, yaw): (f32, f32, f32)| {
            unsafe {
                let world = this.world_mut();
                if let Some(mut transform) = world.get_mut::<Transform>(this.entity) {
                    transform.rotation = Quat::from_euler(EulerRot::XYZ, roll, pitch, yaw);
                }
            }
            Ok(())
        });

        // Get entity velocity (if it has Velocity from rapier)
        methods.add_method("get_velocity", |lua, this, ()| {
            unsafe {
                let world = this.world();
                if let Some(velocity) = world.get::<Velocity>(this.entity) {
                    let table = lua.create_table()?;
                    table.set("x", velocity.linvel.x)?;
                    table.set("y", velocity.linvel.y)?;
                    table.set("angular", velocity.angvel)?;
                    Ok(table)
                } else {
                    lua.create_table()
                }
            }
        });

        // Set entity velocity (if it has Velocity from rapier)
        methods.add_method("set_velocity", |_, this, (x, y): (f32, f32)| {
            unsafe {
                let world = this.world_mut();
                if let Some(mut velocity) = world.get_mut::<Velocity>(this.entity) {
                    velocity.linvel.x = x;
                    velocity.linvel.y = y;
                }
            }
            Ok(())
        });

        // Set entity angular velocity (if it has Velocity from rapier)
        methods.add_method("set_angular_velocity", |_, this, angular: f32| {
            unsafe {
                let world = this.world_mut();
                if let Some(mut velocity) = world.get_mut::<Velocity>(this.entity) {
                    velocity.angvel = angular;
                }
            }
            Ok(())
        });

        // Despawn the entity
        methods.add_method("despawn", |_, this, ()| {
            unsafe {
                let world = this.world_mut();
                if world.get_entity(this.entity).is_ok() {
                    world.despawn(this.entity);
                }
            }
            Ok(())
        });

        // Query nearby entities within a radius
        methods.add_method("query_nearby", |lua, this, radius: f32| {
            unsafe {
                let world = this.world_mut();
                let my_pos = if let Some(transform) = world.get::<Transform>(this.entity) {
                    transform.translation
                } else {
                    return lua.create_table(); // Return empty table if no position
                };

                let results = lua.create_table()?;
                let mut index = 1;

                let mut query = world.query::<(Entity, &Transform)>();
                for (entity, transform) in query.iter(world) {
                    if entity == this.entity {
                        continue; // Skip self
                    }

                    let distance = my_pos.distance(transform.translation);
                    if distance <= radius {
                        let entity_data = lua.create_table()?;
                        entity_data.set("entity", entity.index())?;
                        entity_data.set("distance", distance)?;
                        entity_data.set("x", transform.translation.x)?;
                        entity_data.set("y", transform.translation.y)?;
                        entity_data.set("z", transform.translation.z)?;
                        results.set(index, entity_data)?;
                        index += 1;
                    }
                }

                Ok(results)
            }
        });

        // Spawn a new entity with position, rotation, velocity
        // Returns the entity ID as a number
        // Note: For now this is a simple spawn. In a full implementation,
        // you'd want to queue spawns to avoid mutation during iteration.
        methods.add_method("spawn_entity", |_, this, (x, y, z, yaw): (f32, f32, f32, f32)| {
            unsafe {
                let world = this.world_mut();

                // Spawn basic entity with transform and physics
                let entity = world.spawn((
                    Transform::from_xyz(x, y, z)
                        .with_rotation(Quat::from_rotation_z(yaw)),
                    GlobalTransform::default(),
                    RigidBody::Dynamic,
                    Collider::ball(5.0), // Small default collider
                    Velocity::zero(),
                )).id();

                info!("[Lua Behavior] Spawned entity {:?} at ({:.1}, {:.1}, {:.1})", entity, x, y, z);

                Ok(entity.index())
            }
        });
    }
}

/// Helper to create a Lua userdata from a World reference
pub fn create_world_api_userdata(
    lua: &Lua,
    entity: Entity,
    world: &mut World,
) -> LuaResult<LuaAnyUserData> {
    unsafe {
        let world_api = LuaWorldApi::new(entity, world as *mut World);
        lua.create_userdata(world_api)
    }
}

// Mark as Send + Sync since we control access through the Lua lock
unsafe impl Send for LuaWorldApi {}
unsafe impl Sync for LuaWorldApi {}
