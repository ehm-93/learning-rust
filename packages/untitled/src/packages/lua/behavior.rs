//! Lua Behavior Bridge
//!
//! This module provides LuaBehavior - a wrapper that stores Lua function references
//! and implements the Behavior trait by calling into Lua for lifecycle events.

use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};
use crate::behavior::{Behavior, Params};

/// A behavior implemented in Lua
/// Stores references to Lua functions for each lifecycle hook
pub struct LuaBehavior {
    /// Name of this behavior (for debugging)
    pub name: String,

    /// Shared Lua state (protected by Mutex for Send + Sync)
    lua: Arc<Mutex<Lua>>,

    /// Registry keys for Lua functions (stored in Lua registry to prevent GC)
    init_state_key: Option<LuaRegistryKey>,
    on_spawn_key: Option<LuaRegistryKey>,
    on_update_key: Option<LuaRegistryKey>,
    on_despawn_key: Option<LuaRegistryKey>,
    on_collision_enter_key: Option<LuaRegistryKey>,
    on_collision_stay_key: Option<LuaRegistryKey>,
    on_collision_exit_key: Option<LuaRegistryKey>,

    /// Behavior-specific configuration
    params: Params,

    /// Behavior state (Lua table stored in registry)
    state_key: Option<LuaRegistryKey>,

    /// Entity this behavior is attached to (set during on_spawn)
    entity: Option<Entity>,
}

impl LuaBehavior {
    /// Create a new LuaBehavior from a Lua table definition
    ///
    /// Expected table structure:
    /// ```lua
    /// {
    ///     init_state = function() return {} end,  -- optional
    ///     on_spawn = function(world) end,         -- optional
    ///     on_update = function(world, dt) end,    -- optional
    ///     on_despawn = function(world) end,       -- optional
    ///     on_collision_enter = function(world, other) end,  -- optional
    ///     on_collision_stay = function(world, other) end,   -- optional
    ///     on_collision_exit = function(world, other) end,   -- optional
    ///     config = { ... }  -- optional default config
    /// }
    /// ```
    pub fn new(
        name: String,
        lua: Arc<Mutex<Lua>>,
        definition: LuaTable,
        params: Params,
    ) -> Result<Self, LuaError> {
        let lua_lock = lua.lock().map_err(|e| {
            LuaError::RuntimeError(format!("Failed to lock Lua state: {}", e))
        })?;

        // Extract and store function references in the Lua registry
        let init_state_key = Self::register_function(&lua_lock, &definition, "init_state")?;
        let on_spawn_key = Self::register_function(&lua_lock, &definition, "on_spawn")?;
        let on_update_key = Self::register_function(&lua_lock, &definition, "update")?;
        let on_despawn_key = Self::register_function(&lua_lock, &definition, "on_despawn")?;
        let on_collision_enter_key = Self::register_function(&lua_lock, &definition, "on_collision_enter")?;
        let on_collision_stay_key = Self::register_function(&lua_lock, &definition, "on_collision_stay")?;
        let on_collision_exit_key = Self::register_function(&lua_lock, &definition, "on_collision_exit")?;

        // Initialize state if init_state function exists
        let state_key = if let Some(ref key) = init_state_key {
            let init_fn: LuaFunction = lua_lock.registry_value(key)?;
            let state: LuaTable = init_fn.call(())?;
            Some(lua_lock.create_registry_value(state)?)
        } else {
            // Create empty state table
            let state = lua_lock.create_table()?;
            Some(lua_lock.create_registry_value(state)?)
        };

        // Extract entity from params if present
        let entity = params.get("entity")
            .and_then(|v| match v {
                crate::behavior::ParamValue::EntityId(e) => Some(*e),
                _ => None,
            });

        drop(lua_lock);

        Ok(Self {
            name,
            lua,
            init_state_key,
            on_spawn_key,
            on_update_key,
            on_despawn_key,
            on_collision_enter_key,
            on_collision_stay_key,
            on_collision_exit_key,
            params,
            state_key,
            entity,
        })
    }

    /// Helper to extract a function from a table and store it in the registry
    fn register_function(
        lua: &Lua,
        table: &LuaTable,
        name: &str,
    ) -> Result<Option<LuaRegistryKey>, LuaError> {
        match table.get::<LuaValue>(name)? {
            LuaValue::Function(func) => {
                let key = lua.create_registry_value(func)?;
                Ok(Some(key))
            }
            LuaValue::Nil => Ok(None),
            _ => Err(LuaError::RuntimeError(format!(
                "Behavior field '{}' must be a function or nil",
                name
            ))),
        }
    }

    /// Call a Lua callback with World API access
    /// Returns Ok(()) if the callback is not registered (allowing optional callbacks)
    fn call_lua_callback(
        &self,
        key: &Option<LuaRegistryKey>,
        world: &mut World,
        args: impl IntoLuaMulti,
    ) -> Result<(), LuaError> {
        let Some(reg_key) = key else {
            return Ok(()); // No callback registered, skip silently
        };

        let lua = self.lua.lock().map_err(|e| {
            LuaError::RuntimeError(format!("Failed to lock Lua state: {}", e))
        })?;

        let entity = self.entity.unwrap_or(Entity::PLACEHOLDER);
        let world_api = super::world_api::create_world_api_userdata(&lua, entity, world)?;

        let func: LuaFunction = lua.registry_value(reg_key)?;
        func.call((world_api, args))
    }
}

// Implement Behavior trait
impl Behavior for LuaBehavior {
    fn on_spawn(&mut self, world: &mut World) {
        if self.on_spawn_key.is_some() {
            info!("[LuaBehavior::{}] on_spawn called for entity {:?}", self.name, self.entity);
        }

        if let Err(e) = self.call_lua_callback(&self.on_spawn_key, world, ()) {
            error!("[LuaBehavior::{}] on_spawn error: {}", self.name, e);
        }
    }

    fn on_update(&mut self, world: &mut World, dt: f32) {
        if let Err(e) = self.call_lua_callback(&self.on_update_key, world, dt) {
            error!("[LuaBehavior::{}] on_update error: {}", self.name, e);
        }
    }

    fn on_despawn(&mut self, world: &mut World) {
        if self.on_despawn_key.is_some() {
            info!("[LuaBehavior::{}] on_despawn called", self.name);
        }

        if let Err(e) = self.call_lua_callback(&self.on_despawn_key, world, ()) {
            error!("[LuaBehavior::{}] on_despawn error: {}", self.name, e);
        }
    }

    fn on_collision_enter(&mut self, world: &mut World, other: Entity) {
        if self.on_collision_enter_key.is_some() {
            info!("[LuaBehavior::{}] on_collision_enter with {:?}", self.name, other);
        }

        if let Err(e) = self.call_lua_callback(&self.on_collision_enter_key, world, other.to_bits()) {
            error!("[LuaBehavior::{}] on_collision_enter error: {}", self.name, e);
        }
    }

    fn on_collision_stay(&mut self, world: &mut World, other: Entity) {
        if let Err(e) = self.call_lua_callback(&self.on_collision_stay_key, world, other.to_bits()) {
            error!("[LuaBehavior::{}] on_collision_stay error: {}", self.name, e);
        }
    }

    fn on_collision_exit(&mut self, world: &mut World, other: Entity) {
        if self.on_collision_exit_key.is_some() {
            info!("[LuaBehavior::{}] on_collision_exit with {:?}", self.name, other);
        }

        if let Err(e) = self.call_lua_callback(&self.on_collision_exit_key, world, other.to_bits()) {
            error!("[LuaBehavior::{}] on_collision_exit error: {}", self.name, e);
        }
    }
}

// Implement Send + Sync (required for Behavior)
unsafe impl Send for LuaBehavior {}
unsafe impl Sync for LuaBehavior {}
