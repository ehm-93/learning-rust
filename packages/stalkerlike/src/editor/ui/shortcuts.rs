//! Keyboard shortcuts reference panel

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Resource to track if the shortcuts panel is visible
#[derive(Resource, Default)]
pub struct ShortcutsPanel {
    pub show: bool,
}

impl ShortcutsPanel {
    pub fn toggle(&mut self) {
        self.show = !self.show;
    }

    pub fn close(&mut self) {
        self.show = false;
    }
}

/// System to handle F1 key to toggle shortcuts panel
pub fn handle_shortcuts_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel: ResMut<ShortcutsPanel>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        panel.toggle();
    }
}

/// System to render the keyboard shortcuts reference panel
pub fn shortcuts_panel_ui(
    mut contexts: EguiContexts,
    mut panel: ResMut<ShortcutsPanel>,
) {
    if !panel.show {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut should_close = false;

    egui::Window::new("Keyboard Shortcuts")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;

            // Camera
            ui.heading("Camera");
            ui.separator();
            shortcut_row(ui, "WASD", "Move camera");
            shortcut_row(ui, "Q / E", "Move down / up");
            shortcut_row(ui, "Mouse", "Look around (when locked)");
            shortcut_row(ui, "Left Alt", "Toggle mouse lock");
            shortcut_row(ui, "Middle Mouse", "Temporary mouse lock");
            shortcut_row(ui, "Shift (hold)", "Move faster (4x speed)");
            shortcut_row(ui, "Ctrl (hold)", "Move slower (0.25x speed)");

            ui.add_space(10.0);

            // File Operations
            ui.heading("File Operations");
            ui.separator();
            shortcut_row(ui, "Ctrl+N", "New scene");
            shortcut_row(ui, "Ctrl+O", "Open scene");
            shortcut_row(ui, "Ctrl+S", "Save scene");
            shortcut_row(ui, "Ctrl+Shift+S", "Save scene as");

            ui.add_space(10.0);

            // Selection & Objects
            ui.heading("Selection & Objects");
            ui.separator();
            shortcut_row(ui, "Left Click", "Select object");
            shortcut_row(ui, "Ctrl+Click", "Multi-select (add/remove)");
            shortcut_row(ui, "Escape", "Deselect all / Cancel mode");
            shortcut_row(ui, "Ctrl+D", "Duplicate selected");
            shortcut_row(ui, "Delete", "Delete selected");
            shortcut_row(ui, "Ctrl+G", "Group selected");
            shortcut_row(ui, "Ctrl+Shift+G", "Ungroup selected");

            ui.add_space(10.0);

            // Transform Gizmos
            ui.heading("Transform Gizmos");
            ui.separator();
            shortcut_row(ui, "F", "Cycle gizmo mode (→)");
            shortcut_row(ui, "Shift+F", "Cycle gizmo mode (←)");
            shortcut_row(ui, "O", "Toggle Local/Global space");
            shortcut_row(ui, "G", "Toggle grid snapping");
            shortcut_row(ui, "Shift (drag)", "Move faster (4x speed)");
            shortcut_row(ui, "Ctrl (drag)", "Move slower (0.25x speed)");

            ui.add_space(10.0);

            // Help
            ui.heading("Help");
            ui.separator();
            shortcut_row(ui, "F1", "Show/hide this panel");

            ui.add_space(10.0);

            if ui.button("Close").clicked() {
                should_close = true;
            }
        });

    if should_close {
        panel.close();
    }
}

/// Helper function to render a shortcut row with consistent formatting
fn shortcut_row(ui: &mut egui::Ui, shortcut: &str, description: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(shortcut).monospace().color(egui::Color32::LIGHT_BLUE));
        ui.label("-");
        ui.label(description);
    });
}
