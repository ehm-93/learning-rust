use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::viewport::grid::GridConfig;
use crate::editor::objects::gizmo::GizmoState;
use crate::editor::persistence::SceneFile;

/// Render the status bar at the bottom of the screen
pub fn status_bar_ui(
    mut contexts: EguiContexts,
    grid_config: Res<GridConfig>,
    gizmo_state: Res<GizmoState>,
    scene_file: Res<SceneFile>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::bottom("status_bar")
        .default_height(25.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Transform mode indicator
                let mode_text = format!("Mode: {:?}", gizmo_state.mode);
                ui.colored_label(egui::Color32::LIGHT_BLUE, mode_text);

                ui.separator();

                // Grid snap indicator
                let snap_text = if grid_config.snap_enabled {
                    "Grid Snap: ON"
                } else {
                    "Grid Snap: OFF"
                };
                let snap_color = if grid_config.snap_enabled {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::GRAY
                };
                ui.colored_label(snap_color, snap_text);

                ui.separator();

                // Scene file indicator
                let file_name = scene_file.path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|f| f.to_str())
                    .unwrap_or("untitled");
                let file_text = if scene_file.dirty {
                    format!("{}*", file_name) // Asterisk indicates unsaved changes
                } else {
                    file_name.to_string()
                };
                let file_color = if scene_file.dirty {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::WHITE
                };
                ui.colored_label(file_color, file_text);

                ui.separator();

                // Help text
                ui.label("F: cycle mode | G: toggle snap | Ctrl+S: save | Ctrl+O: load");
            });
        });
}
