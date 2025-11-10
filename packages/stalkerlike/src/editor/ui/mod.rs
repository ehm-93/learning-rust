//! UI domain - Editor user interface panels
//!
//! This domain contains all egui-based UI panels:
//! - Menu bar (File menu with New, Open, Save, Save As)
//! - Asset browser (primitive selection)
//! - Inspector (property editing for selected objects)
//! - Status bar (editor state indicators)
//! - Future: Hierarchy panel, toolbar, settings

pub mod asset_browser;
pub mod inspector;
pub mod menu_bar;
pub mod status_bar;
