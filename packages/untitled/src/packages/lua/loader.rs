use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use super::manifest::{PackageManifest, PackageSource};

/// Scans a directory for packages and loads their manifests
pub struct PackageLoader {
    packages_dir: PathBuf,
}

impl PackageLoader {
    pub fn new(packages_dir: impl Into<PathBuf>) -> Self {
        Self {
            packages_dir: packages_dir.into(),
        }
    }

    /// Scan the packages directory and load all valid packages
    pub fn discover_packages(&self) -> Result<Vec<PackageSource>, Box<dyn std::error::Error>> {
        let mut packages = Vec::new();

        if !self.packages_dir.exists() {
            warn!("Packages directory does not exist: {:?}", self.packages_dir);
            return Ok(packages);
        }

        info!("Scanning for packages in: {:?}", self.packages_dir);

        let entries = fs::read_dir(&self.packages_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only look at directories for now (Phase 1)
            if path.is_dir() {
                match self.load_package_from_dir(&path) {
                    Ok(Some(package)) => {
                        info!("Found package: {} v{}", package.name(), package.version());
                        packages.push(package);
                    }
                    Ok(None) => {
                        // Not a valid package, skip silently
                    }
                    Err(e) => {
                        warn!("Failed to load package from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Try to load a package from a directory
    fn load_package_from_dir(&self, dir: &Path) -> Result<Option<PackageSource>, Box<dyn std::error::Error>> {
        let manifest_path = dir.join("package.toml");
        let init_lua_path = dir.join("init.lua");

        // Check if this is a valid package
        if !manifest_path.exists() {
            return Ok(None);
        }

        if !init_lua_path.exists() {
            return Err(format!("Package at {:?} is missing init.lua", dir).into());
        }

        // Read and parse the manifest
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: PackageManifest = toml::from_str(&manifest_content)?;

        Ok(Some(PackageSource {
            manifest,
            root_path: dir.to_path_buf(),
            init_lua_path,
        }))
    }
}
