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
    api_table.set("behaviors", behaviors_table)?;

    // Set the global 'api' object
    lua.globals().set("api", api_table)?;

    Ok(())
}
