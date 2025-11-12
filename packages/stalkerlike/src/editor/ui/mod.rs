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

pub use plugin::UiPlugin;
pub use hierarchy::{Locked, Hidden};
