use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

/// Represents a single package with its isolated Lua state
/// The Lua state is protected by a Mutex to make it Send + Sync
pub struct LuaPackageState {
    pub name: String,
    pub version: String,
    lua: Arc<Mutex<Lua>>,
}

impl LuaPackageState {
    /// Create a new package state with isolated Lua runtime
    pub fn new(name: String, version: String) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Creating Lua state for package: {} v{}", name, version);

        let lua = Lua::new();

        // Inject the API bridge into this package's state
        inject_api(&lua, &name)?;

        Ok(Self {
            name,
            version,
            lua: Arc::new(Mutex::new(lua)),
        })
    }

    /// Get a reference to the Lua state
    pub fn lua(&self) -> Arc<Mutex<Lua>> {
        Arc::clone(&self.lua)
    }

    /// Execute Lua code in this package's context
    pub fn execute(&self, lua_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        let lua = self.lua.lock().map_err(|e| format!("Failed to lock Lua state: {}", e))?;
        lua.load(lua_code).exec()?;
        Ok(())
    }

    /// Get all registered behavior factories from this package's Lua state
    /// Returns a list of (name, factory_function) pairs
    pub fn get_registered_behaviors(&self) -> Result<Vec<(String, LuaFunction)>, Box<dyn std::error::Error>> {
        let lua = self.lua.lock().map_err(|e| format!("Failed to lock Lua state: {}", e))?;
        let globals = lua.globals();

        // Get the __behaviors__ table if it exists
        let behaviors_storage: LuaTable = match globals.get("__behaviors__") {
            Ok(table) => table,
            Err(_) => return Ok(Vec::new()), // No behaviors registered
        };

        let mut behaviors = Vec::new();

        // Iterate over all behaviors in the storage
        for pair in behaviors_storage.pairs::<String, LuaFunction>() {
            let (name, factory) = pair?;
            behaviors.push((name, factory));
        }

        Ok(behaviors)
    }
}

/// Inject the API bridge into a Lua state
/// This creates a global `api` table with methods like log() and register()
fn inject_api(lua: &Lua, package_name: &str) -> LuaResult<()> {
    let api_table = lua.create_table()?;

    // Add api.log() method
    let pkg_name = package_name.to_string();
    let log_fn = lua.create_function(move |_, msg: String| {
        info!("[Package: {}] {}", pkg_name, msg);
        Ok(())
    })?;
    api_table.set("log", log_fn)?;

    // Create api.behaviors table
    let behaviors_table = lua.create_table()?;

    // Add api.behaviors.register() method
    // Accepts a factory function that takes params and returns a behavior definition table
    let pkg_name = package_name.to_string();
    let behaviors_register_fn = lua.create_function(move |lua, (name, factory): (String, LuaFunction)| {
        info!("[Package: {}] Registering behavior factory: {}", pkg_name, name);

        // Store the factory function in a global table for later retrieval
        let globals = lua.globals();
        let behaviors_storage: LuaTable = match globals.get("__behaviors__") {
            Ok(table) => table,
            Err(_) => {
                let table = lua.create_table()?;
                globals.set("__behaviors__", table.clone())?;
                table
            }
        };

        behaviors_storage.set(name.clone(), factory)?;
        info!("[Package: {}] Behavior factory '{}' registered successfully", pkg_name, name);

        Ok(())
    })?;

    behaviors_table.set("register", behaviors_register_fn)?;

    // Add api.behaviors.create() method
    // Creates a behavior instance by calling a registered factory
    let behaviors_create_fn = lua.create_function(|lua, (name, params): (String, LuaTable)| {
        let globals = lua.globals();
        let behaviors_storage: LuaTable = globals.get("__behaviors__")?;

        // Get the factory function
        let factory: LuaFunction = behaviors_storage.get(name.clone())?;

        // Call the factory with params to create the behavior
        let behavior_table: LuaTable = factory.call(params)?;

        Ok(behavior_table)
    })?;

    behaviors_table.set("create", behaviors_create_fn)?;

    // Add api.behaviors.compose() method
    // Helper to create composite behavior definitions that pass through all lifecycle calls
    // Usage: api.behaviors.compose("my_composite", {"behavior1", "behavior2"})
    // The params will be passed when the behavior is instantiated (like any other behavior)
    let behaviors_compose_fn = lua.create_function(|lua, (name, behavior_names): (String, Vec<String>)| {
        let behavior_count = behavior_names.len();

        // Create a factory function that composes the behaviors
        // This factory accepts params at instantiation time
        let factory = lua.create_function(move |lua, params: LuaTable| {
            let behavior_names = behavior_names.clone();

            // Create the composite behavior table
            let composite = lua.create_table()?;

            // Create instances of all sub-behaviors with the provided params
            let mut sub_behaviors = Vec::new();
            let globals = lua.globals();
            let behaviors_storage: LuaTable = globals.get("__behaviors__")?;

            for behavior_name in &behavior_names {
                let factory: LuaFunction = behaviors_storage.get(behavior_name.clone())?;
                let behavior_table: LuaTable = factory.call(params.clone())?;
                sub_behaviors.push(behavior_table);
            }

            // Store sub-behaviors
            let sub_behaviors_table = lua.create_table()?;
            for (i, behavior) in sub_behaviors.iter().enumerate() {
                sub_behaviors_table.set(i + 1, behavior.clone())?;
            }
            composite.set("__sub_behaviors", sub_behaviors_table.clone())?;

            // on_spawn: call all sub-behaviors
            let sub_behaviors_clone = sub_behaviors_table.clone();
            composite.set("on_spawn", lua.create_function(move |_lua, world: LuaTable| {
                let sub_behaviors: LuaTable = sub_behaviors_clone.clone();
                for pair in sub_behaviors.pairs::<usize, LuaTable>() {
                    let (_, behavior) = pair?;
                    if let Ok(on_spawn) = behavior.get::<LuaFunction>("on_spawn") {
                        on_spawn.call::<()>(world.clone())?;
                    }
                }
                Ok(())
            })?)?;

            // update: call all sub-behaviors
            let sub_behaviors_clone = sub_behaviors_table.clone();
            composite.set("update", lua.create_function(move |_lua, (world, dt): (LuaTable, f32)| {
                let sub_behaviors: LuaTable = sub_behaviors_clone.clone();
                for pair in sub_behaviors.pairs::<usize, LuaTable>() {
                    let (_, behavior) = pair?;
                    if let Ok(update) = behavior.get::<LuaFunction>("update") {
                        update.call::<()>((world.clone(), dt))?;
                    }
                }
                Ok(())
            })?)?;

            // on_despawn: call all sub-behaviors
            let sub_behaviors_clone = sub_behaviors_table.clone();
            composite.set("on_despawn", lua.create_function(move |_lua, world: LuaTable| {
                let sub_behaviors: LuaTable = sub_behaviors_clone.clone();
                for pair in sub_behaviors.pairs::<usize, LuaTable>() {
                    let (_, behavior) = pair?;
                    if let Ok(on_despawn) = behavior.get::<LuaFunction>("on_despawn") {
                        on_despawn.call::<()>(world.clone())?;
                    }
                }
                Ok(())
            })?)?;

            // on_collision_enter: call all sub-behaviors
            let sub_behaviors_clone = sub_behaviors_table.clone();
            composite.set("on_collision_enter", lua.create_function(move |_lua, (world, other): (LuaTable, LuaValue)| {
                let sub_behaviors: LuaTable = sub_behaviors_clone.clone();
                for pair in sub_behaviors.pairs::<usize, LuaTable>() {
                    let (_, behavior) = pair?;
                    if let Ok(handler) = behavior.get::<LuaFunction>("on_collision_enter") {
                        handler.call::<()>((world.clone(), other.clone()))?;
                    }
                }
                Ok(())
            })?)?;

            // on_collision_stay: call all sub-behaviors
            let sub_behaviors_clone = sub_behaviors_table.clone();
            composite.set("on_collision_stay", lua.create_function(move |_lua, (world, other): (LuaTable, LuaValue)| {
                let sub_behaviors: LuaTable = sub_behaviors_clone.clone();
                for pair in sub_behaviors.pairs::<usize, LuaTable>() {
                    let (_, behavior) = pair?;
                    if let Ok(handler) = behavior.get::<LuaFunction>("on_collision_stay") {
                        handler.call::<()>((world.clone(), other.clone()))?;
                    }
                }
                Ok(())
            })?)?;

            // on_collision_exit: call all sub-behaviors
            let sub_behaviors_clone = sub_behaviors_table.clone();
            composite.set("on_collision_exit", lua.create_function(move |_lua, (world, other): (LuaTable, LuaValue)| {
                let sub_behaviors: LuaTable = sub_behaviors_clone.clone();
                for pair in sub_behaviors.pairs::<usize, LuaTable>() {
                    let (_, behavior) = pair?;
                    if let Ok(handler) = behavior.get::<LuaFunction>("on_collision_exit") {
                        handler.call::<()>((world.clone(), other.clone()))?;
                    }
                }
                Ok(())
            })?)?;

            Ok(composite)
        })?;

        // Register the composite behavior factory
        let globals = lua.globals();
        let behaviors_storage: LuaTable = match globals.get("__behaviors__") {
            Ok(table) => table,
            Err(_) => {
                let table = lua.create_table()?;
                globals.set("__behaviors__", table.clone())?;
                table
            }
        };

        behaviors_storage.set(name.clone(), factory)?;
        info!("[Lua] Composite behavior '{}' registered with {} sub-behaviors", name, behavior_count);

        Ok(())
    })?;

    behaviors_table.set("compose", behaviors_compose_fn)?;
    api_table.set("behaviors", behaviors_table)?;

    // Set the global 'api' object
    lua.globals().set("api", api_table)?;

    Ok(())
}
