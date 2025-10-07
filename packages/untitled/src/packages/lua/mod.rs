//! Lua runtime integration for the package system
//!
//! This module provides the Lua state management and API bindings
//! that enable packages to execute Lua code with access to game APIs.

mod state;
mod manifest;
mod loader;

pub use state::*;
pub use manifest::*;
pub use loader::*;
