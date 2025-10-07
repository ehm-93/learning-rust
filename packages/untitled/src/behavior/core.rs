use bevy::prelude::*;
use std::collections::HashMap;

/// Parameter value types that can be passed to behaviors
#[derive(Debug, Clone)]
pub enum ParamValue {
    Float(f32),
    Int(i32),
    Vec3(Vec3),
    String(String),
    EntityId(Entity),
    BehaviorId(String),
}

impl From<f32> for ParamValue {
    fn from(value: f32) -> Self {
        ParamValue::Float(value)
    }
}

impl From<i32> for ParamValue {
    fn from(value: i32) -> Self {
        ParamValue::Int(value)
    }
}

impl From<Vec3> for ParamValue {
    fn from(value: Vec3) -> Self {
        ParamValue::Vec3(value)
    }
}

impl From<String> for ParamValue {
    fn from(value: String) -> Self {
        ParamValue::String(value)
    }
}

impl From<&str> for ParamValue {
    fn from(value: &str) -> Self {
        ParamValue::String(value.to_string())
    }
}

impl From<Entity> for ParamValue {
    fn from(value: Entity) -> Self {
        ParamValue::EntityId(value)
    }
}

/// Parameters passed to behaviors - maps parameter names to values
#[derive(Debug, Clone, Default)]
pub struct Params {
    inner: HashMap<String, ParamValue>,
}

impl Params {
    /// Create a new empty Params instance
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Insert a parameter
    pub fn insert(&mut self, key: impl Into<String>, value: ParamValue) {
        self.inner.insert(key.into(), value);
    }

    /// Get a parameter by key
    pub fn get(&self, key: &str) -> Option<&ParamValue> {
        self.inner.get(key)
    }

    /// Get a float parameter
    pub fn get_f32(&self, key: &str) -> Result<f32, String> {
        match self.get(key) {
            Some(ParamValue::Float(value)) => Ok(*value),
            Some(ParamValue::Int(value)) => Ok(*value as f32),
            Some(_) => Err(format!("Parameter '{}' is not a numeric type", key)),
            None => Err(format!("Parameter '{}' not found", key)),
        }
    }

    /// Get an integer parameter
    pub fn get_int(&self, key: &str) -> Result<i32, String> {
        match self.get(key) {
            Some(ParamValue::Int(value)) => Ok(*value),
            Some(ParamValue::Float(value)) => Ok(*value as i32),
            Some(_) => Err(format!("Parameter '{}' is not a numeric type", key)),
            None => Err(format!("Parameter '{}' not found", key)),
        }
    }

    /// Get a Vec3 parameter
    pub fn get_vec3(&self, key: &str) -> Result<Vec3, String> {
        match self.get(key) {
            Some(ParamValue::Vec3(value)) => Ok(*value),
            Some(_) => Err(format!("Parameter '{}' is not a Vec3", key)),
            None => Err(format!("Parameter '{}' not found", key)),
        }
    }

    /// Get a string parameter
    pub fn get_string(&self, key: &str) -> Result<String, String> {
        match self.get(key) {
            Some(ParamValue::String(value)) => Ok(value.clone()),
            Some(_) => Err(format!("Parameter '{}' is not a string", key)),
            None => Err(format!("Parameter '{}' not found", key)),
        }
    }

    /// Get an entity parameter
    pub fn get_entity(&self, key: &str) -> Result<Entity, String> {
        match self.get(key) {
            Some(ParamValue::EntityId(value)) => Ok(*value),
            Some(_) => Err(format!("Parameter '{}' is not an entity", key)),
            None => Err(format!("Parameter '{}' not found", key)),
        }
    }

    /// Get a behavior ID parameter
    pub fn get_behavior_id(&self, key: &str) -> Result<String, String> {
        match self.get(key) {
            Some(ParamValue::BehaviorId(value)) => Ok(value.clone()),
            Some(ParamValue::String(value)) => Ok(value.clone()),
            Some(_) => Err(format!("Parameter '{}' is not a behavior ID", key)),
            None => Err(format!("Parameter '{}' not found", key)),
        }
    }

    /// Get a subset of parameters with the specified keys
    pub fn subset(&self, keys: &[&str]) -> Params {
        let mut result = Params::new();
        for key in keys {
            if let Some(value) = self.get(*key) {
                result.insert(key.to_string(), value.clone());
            }
        }
        result
    }

    /// Create Params from an array of key-value pairs
    pub fn from<const N: usize>(params: [(&str, ParamValue); N]) -> Self {
        let mut params_obj = Params::new();
        for (key, value) in params {
            params_obj.insert(key, value);
        }
        params_obj
    }

    /// Iterate over all parameters
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParamValue)> {
        self.inner.iter()
    }
}

/// The core behavior trait - all behaviors implement this
pub trait Behavior: Send + Sync {
    /// Set the entity this behavior is attached to
    fn set_entity(&mut self, _entity: Entity) {}

    /// Called when the behavior is first spawned on an entity
    fn on_spawn(&mut self, _world: &mut World) {}

    /// Called every frame while the behavior is active
    fn on_update(&mut self, _world: &mut World, _dt: f32) {}

    /// Called when the behavior is about to be despawned
    fn on_despawn(&mut self, _world: &mut World) {}

    /// Called when the entity enters collision with another entity
    fn on_collision_enter(&mut self, _world: &mut World, _other: Entity) {}

    /// Called while the entity is in collision with another entity
    fn on_collision_stay(&mut self, _world: &mut World, _other: Entity) {}

    /// Called when the entity exits collision with another entity
    fn on_collision_exit(&mut self, _world: &mut World, _other: Entity) {}
}

/// Type alias for behavior definition functions
/// Changed to Box<dyn Fn> to support closures (needed for Lua behaviors)
pub type BehaviorDefinition = Box<dyn Fn(Params) -> Box<dyn Behavior> + Send + Sync>;
