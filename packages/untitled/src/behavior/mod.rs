// Behavior System Module
//
// This module implements a composable behavior system for Bevy ECS with Lua integration support.
// Behaviors are lifecycle-aware components that can be atomic or composite, supporting both
// compile-time tuple composition (Rust) and runtime dynamic composition (Lua).

pub mod core;
pub mod registry;
pub mod component;
pub mod composition;
pub mod systems;
pub mod plugin;

// Re-export core types
pub use core::*;
pub use registry::*;
pub use component::*;
pub use composition::*;
pub use systems::*;
pub use plugin::*;
