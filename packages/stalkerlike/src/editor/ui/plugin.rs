//! UI plugin for editor user interface

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

// Note: asset_browser_ui is now integrated into hierarchy_ui
use super::confirmation_dialog::{
    ConfirmationDialog, ErrorDialog, AutoSaveRecoveryDialog,
    confirmation_dialog_ui, error_dialog_ui, autosave_recovery_dialog_ui
};
use super::hierarchy::{
    hierarchy_ui, HierarchyState, AssetBrowserState,
    handle_directory_picker, poll_directory_picker_tasks
};
use super::inspector::{inspector_ui, InspectorState};
use super::menu_bar::menu_bar_ui;
use super::shortcuts::{shortcuts_panel_ui, handle_shortcuts_key, ShortcutsPanel};
use super::status_bar::status_bar_ui;

/// Plugin for all editor UI panels (menu bar, hierarchy+assets, inspector, etc.)
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Third-party plugins
            .add_plugins(EguiPlugin::default())

            // Resources
            .init_resource::<HierarchyState>()
            .init_resource::<AssetBrowserState>()
            .init_resource::<InspectorState>()
            .init_resource::<ConfirmationDialog>()
            .init_resource::<ErrorDialog>()
            .init_resource::<AutoSaveRecoveryDialog>()
            .init_resource::<ShortcutsPanel>()

            // UI systems (must run in EGUI pass)
            .add_systems(EguiPrimaryContextPass, (
                menu_bar_ui,
                confirmation_dialog_ui,
                error_dialog_ui,
                autosave_recovery_dialog_ui,
                shortcuts_panel_ui,
                status_bar_ui,
                hierarchy_ui,      // Now includes asset browser
                inspector_ui,
            ).chain())

            // Background systems for file dialogs
            .add_systems(Update, (
                handle_directory_picker,
                poll_directory_picker_tasks,
                handle_shortcuts_key,
            ));
    }
}
