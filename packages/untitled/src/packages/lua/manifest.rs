use serde::Deserialize;
use std::path::PathBuf;

/// Package manifest from package.toml
#[derive(Debug, Clone, Deserialize)]
pub struct PackageManifest {
    pub package: PackageInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Represents a package found on disk
#[derive(Debug, Clone)]
pub struct PackageSource {
    pub manifest: PackageManifest,
    pub root_path: PathBuf,
    pub init_lua_path: PathBuf,
}

impl PackageSource {
    pub fn name(&self) -> &str {
        &self.manifest.package.name
    }

    pub fn version(&self) -> &str {
        &self.manifest.package.version
    }
}
