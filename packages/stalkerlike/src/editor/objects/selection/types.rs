//! Selection types and resources
//!
//! This module defines the data structures for tracking selected entities:
//! - `SelectionSet`: Resource tracking the set of currently selected entities
//! - `Selected`: Marker component for selected entities
//! - `SelectedEntity`: Legacy resource for backward compatibility

use bevy::prelude::*;
use std::collections::HashSet;

/// Resource tracking the set of currently selected entities
#[derive(Resource, Default)]
pub struct SelectionSet {
    pub entities: HashSet<Entity>,
}

impl SelectionSet {
    /// Check if an entity is selected
    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    /// Add an entity to the selection
    pub fn add(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    /// Remove an entity from the selection
    pub fn remove(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }

    /// Toggle an entity's selection state
    pub fn toggle(&mut self, entity: Entity) {
        if self.entities.contains(&entity) {
            self.entities.remove(&entity);
        } else {
            self.entities.insert(entity);
        }
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Get the first selected entity (for backward compatibility)
    pub fn first(&self) -> Option<Entity> {
        self.entities.iter().next().copied()
    }

    /// Check if the selection is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get the number of selected entities
    pub fn len(&self) -> usize {
        self.entities.len()
    }
}

// Legacy alias for backward compatibility - will be removed after migration
#[derive(Resource, Default)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
}

/// Marker component for selected entities
#[derive(Component)]
pub struct Selected;
