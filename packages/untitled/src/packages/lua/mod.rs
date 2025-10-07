//! Lua runtime integration for the package system
//!
//! This module provides the Lua state management and API bindings
//! that enable packages to execute Lua code with access to game APIs.

mod state;
mod manifest;
mod loader;
mod behavior;
mod world_api;

// Public API - only expose what's needed by the plugin
pub use state::LuaPackageState;
pub use loader::PackageLoader;
pub use behavior::LuaBehavior;
