//! UI plugin for editor user interface

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use super::asset_browser::asset_browser_ui;
use super::confirmation_dialog::{ConfirmationDialog, ErrorDialog, confirmation_dialog_ui, error_dialog_ui};
use super::inspector::{inspector_ui, InspectorState};
use super::menu_bar::menu_bar_ui;
use super::status_bar::status_bar_ui;

/// Plugin for all editor UI panels (menu bar, asset browser, inspector, etc.)
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Third-party plugins
            .add_plugins(EguiPlugin::default())

            // Resources
            .init_resource::<InspectorState>()
            .init_resource::<ConfirmationDialog>()
            .init_resource::<ErrorDialog>()

            // UI systems (must run in EGUI pass)
            .add_systems(EguiPrimaryContextPass, (
                menu_bar_ui,
                confirmation_dialog_ui,
                error_dialog_ui,
                status_bar_ui,
                asset_browser_ui,
                inspector_ui,
            ).chain());
    }
}
