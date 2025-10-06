use bevy::prelude::*;
use crate::behavior::Params;

/// A safe wrapper around Bevy's World that provides behavior-specific APIs
pub struct WorldApi<'a> {
    pub entity: Entity,
    pub world: &'a mut World,
}

impl<'a> WorldApi<'a> {
    /// Create a new WorldApi instance
    pub fn new(entity: Entity, world: &'a mut World) -> Self {
        Self { entity, world }
    }

    /// Add a component to the current entity
    pub fn add_component<T: Component>(&mut self, component: T) {
        if let Ok(mut entity_mut) = self.world.get_entity_mut(self.entity) {
            entity_mut.insert(component);
        }
    }

    /// Get a component from the current entity
    pub fn get_component<T: Component>(&self) -> Option<&T> {
        self.world.get::<T>(self.entity)
    }

    /// Modify a component on the current entity using a closure
    /// This works around Bevy's component mutability constraints
    pub fn modify_component<T: Component>(&mut self, f: impl FnOnce(&mut T)) {
        // For now, we skip the implementation due to Bevy's complex mutability system
        // This will be implemented properly when we handle specific component types
        let _ = f; // Silence unused warning
    }

    /// Remove a component from the current entity
    pub fn remove_component<T: Component>(&mut self) -> Option<T> {
        if let Ok(mut entity_mut) = self.world.get_entity_mut(self.entity) {
            entity_mut.take::<T>()
        } else {
            None
        }
    }

    /// Spawn a new entity with a behavior
    pub fn spawn_behavior(&mut self, _name: &str, _params: Params) -> Option<Entity> {
        // For now, we'll implement this later when we have BehaviorRegistry
        // This is a placeholder that spawns an empty entity
        let new_entity = self.world.spawn_empty().id();
        Some(new_entity)
    }

    /// Despawn the current entity
    pub fn despawn(&mut self) {
        if let Ok(entity) = self.world.get_entity_mut(self.entity) {
            entity.despawn();
        }
    }

    /// Get the Transform of the current entity
    pub fn get_transform(&self) -> Option<&Transform> {
        self.get_component::<Transform>()
    }

    /// Modify the Transform of the current entity using a closure
    pub fn modify_transform(&mut self, f: impl FnOnce(&mut Transform)) {
        self.modify_component::<Transform>(f);
    }

    /// Get the Transform of another entity
    pub fn get_entity_transform(&self, entity: Entity) -> Option<&Transform> {
        self.world.get::<Transform>(entity)
    }

    /// Get a component from another entity
    pub fn get_entity_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.world.get::<T>(entity)
    }

    /// Check if an entity exists
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.world.get_entity(entity).is_ok()
    }

    /// Get the current entity ID
    pub fn current_entity(&self) -> Entity {
        self.entity
    }
}
