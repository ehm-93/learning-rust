//! Package loading system for Phase 0 Lua integration
//!
//! This module implements the foundation for the effect-behavior package system.
//! Phase 0 provides:
//! - mlua integrated with Bevy
//! - Isolated Lua states per package
//! - Global `api` object injected into each state
//! - Basic API methods: `api.log()`, `api.register()`

pub mod lua;
mod plugin;
mod behaviors;  // Legacy behavior system, will be replaced by package system

pub use plugin::*;
pub use behaviors::*;  // Re-export for backward compatibility
