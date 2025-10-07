use bevy::prelude::*;
use std::collections::HashMap;
use crate::behavior::{Behavior, BehaviorDefinition, Params};

/// BehaviorSpec represents a single behavior instantiation with its parameters
#[derive(Debug, Clone)]
pub struct BehaviorSpec {
    pub name: String,
    pub params: Params,
}

impl BehaviorSpec {
    pub fn new(name: impl Into<String>, params: Params) -> Self {
        Self {
            name: name.into(),
            params,
        }
    }
}

/// Registry for behavior definitions - stored as a Bevy Resource
#[derive(Resource)]
pub struct BehaviorRegistry {
    /// Maps behavior names to factory functions that create behaviors
    behaviors: HashMap<String, BehaviorDefinition>,
}

impl BehaviorRegistry {
    /// Create a new empty behavior registry
    pub fn new() -> Self {
        Self {
            behaviors: HashMap::new(),
        }
    }

    /// Register a single behavior definition
    pub fn register(&mut self, name: &str, def: BehaviorDefinition) {
        self.behaviors.insert(name.to_string(), def);
    }

    pub fn get(&self, name: &str) -> Option<&BehaviorDefinition> {
        self.behaviors.get(name)
    }

    /// Instantiate a behavior by name, returning the behavior
    pub fn instantiate(&self, name: &str, params: Params) -> Option<Box<dyn Behavior>> {
        if let Some(def) = self.behaviors.get(name) {
            Some((def)(params))
        } else {
            None
        }
    }

    /// Check if a behavior is registered
    pub fn has_behavior(&self, name: &str) -> bool {
        self.behaviors.contains_key(name)
    }

    /// Get a list of all registered behavior names
    pub fn list_behaviors(&self) -> Vec<String> {
        self.behaviors.keys().cloned().collect()
    }
}

impl Default for BehaviorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
