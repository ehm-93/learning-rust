use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::objects::selection::SelectedEntity;

/// Render the inspector panel
pub fn inspector_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedEntity>,
    transform_query: Query<&Transform>,
    name_query: Query<&Name>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Inspector")
        .default_pos([1200.0, 100.0])
        .default_width(200.0)
        .show(ctx, |ui| {
            if let Some(entity) = selected.entity {
                ui.heading("Selected Object");
                ui.separator();

                // Show entity name if it has one
                if let Ok(name) = name_query.get(entity) {
                    ui.label(format!("Name: {}", name));
                } else {
                    ui.label(format!("Entity: {:?}", entity));
                }

                // Show transform (read-only for now)
                if let Ok(transform) = transform_query.get(entity) {
                    ui.label("Transform:");
                    ui.group(|ui| {
                        ui.label(format!("X: {:.2}", transform.translation.x));
                        ui.label(format!("Y: {:.2}", transform.translation.y));
                        ui.label(format!("Z: {:.2}", transform.translation.z));
                    });
                }
            } else {
                ui.label("No selection");
                ui.separator();
                ui.label("Click an object to select");
            }
        });
}
