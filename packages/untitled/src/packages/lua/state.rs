use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

/// Represents a single package with its isolated Lua state
/// The Lua state is protected by a Mutex to make it Send + Sync
#[derive(Debug)]
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

    /// Execute Lua code in this package's context
    pub fn execute(&self, lua_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        let lua = self.lua.lock().map_err(|e| format!("Failed to lock Lua state: {}", e))?;
        lua.load(lua_code).exec()?;
        Ok(())
    }

    /// Execute a Lua function by name (simplified for Phase 0)
    pub fn call_function(&self, function_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let lua = self.lua.lock().map_err(|e| format!("Failed to lock Lua state: {}", e))?;
        let globals = lua.globals();
        let function: LuaFunction = globals.get(function_name)?;
        function.call::<()>(())?;
        Ok(())
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

    // Add api.register() method - for Phase 0 this just logs the registration
    let pkg_name = package_name.to_string();
    let register_fn = lua.create_function(move |_, (name, data): (String, LuaValue)| {
        info!("[Package: {}] Registered '{}': {:?}", pkg_name, name, data);
        Ok(())
    })?;
    api_table.set("register", register_fn)?;

    // Set the global 'api' object
    lua.globals().set("api", api_table)?;

    Ok(())
}
