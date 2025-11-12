//! UI domain - Editor user interface panels
//!
//! This domain contains all egui-based UI panels:
//! - Menu bar (File menu with New, Open, Save, Save As)
//! - Asset browser (primitive selection)
//! - Hierarchy panel (scene entity tree view)
//! - Inspector (property editing for selected objects)
//! - Status bar (editor state indicators)
//! - Confirmation dialog (unsaved changes prompt)
//! - Future: Toolbar, settings

pub mod asset_browser;
pub mod confirmation_dialog;
pub mod hierarchy;
pub mod inspector;
pub mod menu_bar;
pub mod plugin;
pub mod shortcuts;
pub mod status_bar;

// Re-export plugin
pub use plugin::UiPlugin;

// Re-export commonly used components/types for other modules
pub use hierarchy::{Locked, Hidden};
