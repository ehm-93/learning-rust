use bevy::prelude::*;
use crate::behavior::Behavior;

/// Component to store behavior instances on an entity
#[derive(Component)]
pub struct BehaviorComponent {
    behaviors: Vec<Box<dyn Behavior>>,
}

impl BehaviorComponent {
    /// Create a new BehaviorComponent with the given behaviors
    pub fn new(behaviors: Vec<Box<dyn Behavior>>) -> Self {
        Self { behaviors }
    }

    /// Set the entity for all behaviors
    pub fn set_entity(&mut self, entity: Entity) {
        for behavior in &mut self.behaviors {
            behavior.set_entity(entity);
        }
    }

    /// Call on_spawn for all behaviors
    pub fn on_spawn(&mut self, world: &mut World) {
        for behavior in &mut self.behaviors {
            behavior.on_spawn(world);
        }
    }

    /// Call on_update for all behaviors
    pub fn on_update(&mut self, world: &mut World, dt: f32) {
        for behavior in &mut self.behaviors {
            behavior.on_update(world, dt);
        }
    }

    /// Call on_despawn for all behaviors
    pub fn on_despawn(&mut self, world: &mut World) {
        for behavior in &mut self.behaviors {
            behavior.on_despawn(world);
        }
    }

    /// Call on_collision_enter for all behaviors
    pub fn on_collision_enter(&mut self, world: &mut World, other: Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_enter(world, other);
        }
    }

    /// Call on_collision_stay for all behaviors
    pub fn on_collision_stay(&mut self, world: &mut World, other: Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_stay(world, other);
        }
    }

    /// Call on_collision_exit for all behaviors
    pub fn on_collision_exit(&mut self, world: &mut World, other: Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_exit(world, other);
        }
    }

    /// Get the number of behaviors in this component
    pub fn behavior_count(&self) -> usize {
        self.behaviors.len()
    }

    /// Check if this component has any behaviors
    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }
}
