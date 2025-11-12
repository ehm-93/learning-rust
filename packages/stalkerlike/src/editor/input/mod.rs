//! Input domain - Input handling and action mapping
//!
//! This domain provides an abstraction layer for input handling:
//! - Mouse state tracking (motion, button states)
//! - Future: Keyboard shortcut system
//! - Future: Configurable keybindings
//! - Future: High-level editor actions (e.g., "ToggleGridSnap", "Delete")

pub mod mouse;
pub mod plugin;

// Re-export plugin
pub use plugin::InputPlugin;

// Re-export commonly used types for other modules
pub use mouse::EditorMouseMotion;
