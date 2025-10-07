use bevy::prelude::*;
use std::collections::HashMap;

/// Simple package metadata for Phase 0
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

/// Resource that manages all loaded packages
/// For Phase 0, we'll just track package metadata and defer Lua execution to runtime
#[derive(Resource, Default)]
pub struct PackageManager {
    packages: HashMap<String, PackageInfo>,
}

impl PackageManager {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Create and register a new package
    pub fn create_package(&mut self, name: String, version: String) -> Result<(), Box<dyn std::error::Error>> {
        let package_info = PackageInfo {
            name: name.clone(),
            version,
        };
        self.packages.insert(name, package_info);
        Ok(())
    }

    /// Get a reference to a package info
    pub fn get_package(&self, package_name: &str) -> Option<&PackageInfo> {
        self.packages.get(package_name)
    }

    /// List all loaded packages
    pub fn list_packages(&self) -> Vec<(&String, &String)> {
        self.packages.iter().map(|(name, info)| (name, &info.version)).collect()
    }
}

/// Bevy plugin for the package system
pub struct PackagePlugin;

impl Plugin for PackagePlugin {
    fn build(&self, app: &mut App) {
        println!("🔧 PackagePlugin::build() called - Phase 0 package system initializing");
        app.insert_resource(PackageManager::new())
           .add_systems(Startup, setup_package_system);
    }
}

/// System to set up the package system during startup
pub fn setup_package_system(
    mut package_manager: ResMut<PackageManager>,
    mut behavior_registry: ResMut<crate::behavior::BehaviorRegistry>,
) {
    println!("🚀 PackagePlugin::setup_package_system called!");
    info!("Initializing package system...");

    // Register built-in Rust behaviors first
    info!("📚 Registering built-in behaviors...");
    crate::behavior::register_builtin_behaviors(&mut behavior_registry);
    info!("✅ Built-in behaviors registered");

    // Phase 1: Load packages from disk
    use super::lua::{PackageLoader, LuaPackageState, LuaBehavior};
    use crate::behavior::Params;
    use std::fs;

    let packages_dir = "packages/untitled/packages";
    let loader = PackageLoader::new(packages_dir);

    match loader.discover_packages() {
        Ok(package_sources) => {
            let package_count = package_sources.len();
            info!("📦 Found {} package(s)", package_count);

            for source in package_sources {
                let name = source.name().to_string();
                let version = source.version().to_string();

                info!("Loading package: {} v{}", name, version);

                // Register the package
                if let Err(e) = package_manager.create_package(name.clone(), version.clone()) {
                    error!("Failed to register package {}: {}", name, e);
                    continue;
                }

                // Load and execute init.lua
                match fs::read_to_string(&source.init_lua_path) {
                    Ok(init_lua_code) => {
                        // Create a Lua state for this package
                        match LuaPackageState::new(name.clone(), version.clone()) {
                            Ok(lua_state) => {
                                if let Err(e) = lua_state.execute(&init_lua_code) {
                                    error!("Failed to execute init.lua for {}: {}", name, e);
                                } else {
                                    info!("✅ Successfully executed init.lua for package: {}", name);

                                    // Phase 2: Extract and register behavior factories
                                    match lua_state.get_registered_behaviors() {
                                        Ok(behaviors) => {
                                            info!("📝 Registering {} behavior(s) from {}", behaviors.len(), name);

                                            let lua = lua_state.lua();
                                            for (behavior_name, factory_fn) in behaviors {
                                                info!("  → Registering behavior: {}", behavior_name);

                                                // Store factory function in registry to prevent GC
                                                let factory_key = {
                                                    let lua_lock = lua.lock().unwrap();
                                                    lua_lock.create_registry_value(factory_fn).unwrap()
                                                };

                                                // Create a Rust factory function that calls the Lua factory
                                                let lua_clone = lua.clone();
                                                let def_name = behavior_name.clone();

                                                let factory: crate::behavior::BehaviorDefinition = Box::new(move |params: Params| {
                                                    // Call the Lua factory function with params
                                                    let lua_lock = lua_clone.lock().unwrap();

                                                    // Convert params to Lua table
                                                    let params_table = lua_lock.create_table().unwrap();

                                                    // Add entity_id to params if present
                                                    if let Some(crate::behavior::ParamValue::EntityId(entity)) = params.get("entity") {
                                                        params_table.set("entity_id", entity.to_bits()).unwrap();
                                                    }

                                                    // Get factory from registry and call it
                                                    let factory_fn: mlua::Function = lua_lock.registry_value(&factory_key).unwrap();
                                                    let definition: mlua::Table = factory_fn.call(params_table).unwrap();
                                                    drop(lua_lock);

                                                    // Create LuaBehavior from the returned definition
                                                    match LuaBehavior::new(
                                                        def_name.clone(),
                                                        lua_clone.clone(),
                                                        definition,
                                                        params,
                                                    ) {
                                                        Ok(behavior) => Box::new(behavior) as Box<dyn crate::behavior::Behavior>,
                                                        Err(e) => {
                                                            error!("Failed to create LuaBehavior: {}", e);
                                                            panic!("Failed to create LuaBehavior: {}", e);
                                                        }
                                                    }
                                                });

                                                behavior_registry.register(&behavior_name, factory);
                                                info!("  ✓ Behavior '{}' registered", behavior_name);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to get registered behaviors from {}: {}", name, e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to create Lua state for {}: {}", name, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read init.lua for {}: {}", name, e);
                    }
                }
            }

            if package_count == 0 {
                warn!("No packages found in {}/", packages_dir);
                println!("⚠️  No packages found. Create a test package at {}/test/", packages_dir);
            } else {
                println!("✅ Phase 1-2 success: Loaded {} package(s) with behaviors", package_count);
            }
        }
        Err(e) => {
            error!("Failed to discover packages: {}", e);
            println!("❌ Failed to discover packages: {}", e);
        }
    }
}
