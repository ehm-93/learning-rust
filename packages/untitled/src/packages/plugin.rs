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
fn setup_package_system(mut package_manager: ResMut<PackageManager>) {
    println!("🚀 PackagePlugin::setup_package_system called!");
    info!("Initializing package system...");

    // Phase 1: Load packages from disk
    use super::lua::{PackageLoader, LuaPackageState};
    use std::fs;

    let packages_dir = "packages";
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
                                    info!("✅ Successfully loaded package: {}", name);
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
                println!("✅ Phase 1 success: Loaded {} package(s) from disk", package_count);
            }
        }
        Err(e) => {
            error!("Failed to discover packages: {}", e);
            println!("❌ Failed to discover packages: {}", e);
        }
    }
}
