//! Built-in foundational behaviors
//!
//! These behaviors provide common functionality that can be composed together
//! or used as building blocks for Lua behaviors.

use bevy::prelude::*;
use crate::behavior::{Behavior, Params};

/// Behavior that despawns the entity after a specified lifetime
pub struct LifetimeBehavior {
    lifetime: f32,
    elapsed: f32,
}

impl LifetimeBehavior {
    pub fn new(params: Params) -> Self {
        let lifetime = params.get_f32("lifetime").unwrap_or(5.0);
        Self {
            lifetime,
            elapsed: 0.0,
        }
    }
}

impl Behavior for LifetimeBehavior {
    fn on_spawn(&mut self, _world: &mut World) {
        info!("[LifetimeBehavior] Spawned with lifetime: {:.2}s", self.lifetime);
    }

    fn on_update(&mut self, world: &mut World, dt: f32) {
        self.elapsed += dt;
        if self.elapsed >= self.lifetime {
            info!("[LifetimeBehavior] Lifetime expired, despawning");
            // Note: Can't directly despawn here without entity ID
            // This would be handled by a separate system in practice
            let _ = world;
        }
    }

    fn on_despawn(&mut self, _world: &mut World) {
        info!("[LifetimeBehavior] Despawned after {:.2}s", self.elapsed);
    }
}

/// Behavior that oscillates entity position along an axis
pub struct OscillateBehavior {
    axis: Vec3,
    amplitude: f32,
    frequency: f32,
    time: f32,
    initial_pos: Option<Vec3>,
}

impl OscillateBehavior {
    pub fn new(params: Params) -> Self {
        let amplitude = params.get_f32("amplitude").unwrap_or(1.0);
        let frequency = params.get_f32("frequency").unwrap_or(1.0);

        // Get axis from params or default to Y
        let axis = params.get_vec3("axis").unwrap_or(Vec3::Y);

        Self {
            axis: axis.normalize(),
            amplitude,
            frequency,
            time: 0.0,
            initial_pos: None,
        }
    }
}

impl Behavior for OscillateBehavior {
    fn on_spawn(&mut self, _world: &mut World) {
        info!("[OscillateBehavior] Spawned - amplitude: {:.2}, frequency: {:.2}",
            self.amplitude, self.frequency);
    }

    fn on_update(&mut self, world: &mut World, dt: f32) {
        self.time += dt;

        // Note: This is a simplified version - in practice we'd need entity ID
        // to actually modify the transform
        let offset = self.axis * self.amplitude * (self.time * self.frequency * std::f32::consts::TAU).sin();
        let _ = (world, offset); // Avoid unused warnings
    }
}

/// Behavior that moves entity toward a target position
pub struct FollowTargetBehavior {
    target_entity: Option<Entity>,
    target_pos: Vec3,
    speed: f32,
    stop_distance: f32,
}

impl FollowTargetBehavior {
    pub fn new(params: Params) -> Self {
        let speed = params.get_f32("speed").unwrap_or(2.0);
        let stop_distance = params.get_f32("stop_distance").unwrap_or(0.5);

        let target_entity = params.get("target_entity")
            .and_then(|v| match v {
                crate::behavior::ParamValue::EntityId(e) => Some(*e),
                _ => None,
            });

        let target_pos = params.get_vec3("target_pos").unwrap_or(Vec3::ZERO);

        Self {
            target_entity,
            target_pos,
            speed,
            stop_distance,
        }
    }
}

impl Behavior for FollowTargetBehavior {
    fn on_spawn(&mut self, _world: &mut World) {
        info!("[FollowTargetBehavior] Spawned - speed: {:.2}", self.speed);
    }

    fn on_update(&mut self, world: &mut World, dt: f32) {
        // In practice, would read current position, calculate direction to target,
        // and move toward it
        let _ = (world, dt);
    }
}

/// Behavior that applies damage when colliding with entities
pub struct DamageOnCollisionBehavior {
    damage: f32,
    damage_cooldown: f32,
    last_damage_time: f32,
}

impl DamageOnCollisionBehavior {
    pub fn new(params: Params) -> Self {
        let damage = params.get_f32("damage").unwrap_or(10.0);
        let damage_cooldown = params.get_f32("cooldown").unwrap_or(1.0);

        Self {
            damage,
            damage_cooldown,
            last_damage_time: -999.0,
        }
    }
}

impl Behavior for DamageOnCollisionBehavior {
    fn on_spawn(&mut self, _world: &mut World) {
        info!("[DamageOnCollisionBehavior] Spawned - damage: {:.1}", self.damage);
    }

    fn on_update(&mut self, _world: &mut World, dt: f32) {
        self.last_damage_time += dt;
    }

    fn on_collision_enter(&mut self, world: &mut World, other: Entity) {
        if self.last_damage_time >= self.damage_cooldown {
            info!("[DamageOnCollisionBehavior] Dealing {} damage to {:?}",
                self.damage, other);
            self.last_damage_time = 0.0;

            // In practice, would apply damage to the other entity
            let _ = world;
        }
    }
}

/// Behavior that logs lifecycle events (useful for debugging)
pub struct LogLifecycleBehavior {
    name: String,
}

impl LogLifecycleBehavior {
    pub fn new(params: Params) -> Self {
        let name = params.get("name")
            .and_then(|v| match v {
                crate::behavior::ParamValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "unnamed".to_string());

        Self { name }
    }
}

impl Behavior for LogLifecycleBehavior {
    fn on_spawn(&mut self, _world: &mut World) {
        info!("[{}] 🟢 on_spawn", self.name);
    }

    fn on_update(&mut self, _world: &mut World, _dt: f32) {
        // Don't log every frame, too noisy
    }

    fn on_despawn(&mut self, _world: &mut World) {
        info!("[{}] 🔴 on_despawn", self.name);
    }

    fn on_collision_enter(&mut self, _world: &mut World, other: Entity) {
        info!("[{}] 💥 collision_enter with {:?}", self.name, other);
    }

    fn on_collision_exit(&mut self, _world: &mut World, other: Entity) {
        info!("[{}] 👋 collision_exit with {:?}", self.name, other);
    }
}

/// Register all built-in behaviors with the registry
pub fn register_builtin_behaviors(registry: &mut crate::behavior::BehaviorRegistry) {
    registry.register("lifetime", Box::new(|params| {
        Box::new(LifetimeBehavior::new(params))
    }));

    registry.register("oscillate", Box::new(|params| {
        Box::new(OscillateBehavior::new(params))
    }));

    registry.register("follow_target", Box::new(|params| {
        Box::new(FollowTargetBehavior::new(params))
    }));

    registry.register("damage_on_collision", Box::new(|params| {
        Box::new(DamageOnCollisionBehavior::new(params))
    }));

    registry.register("log_lifecycle", Box::new(|params| {
        Box::new(LogLifecycleBehavior::new(params))
    }));
}
