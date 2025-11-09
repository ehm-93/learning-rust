use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::placement::{start_placement, PlacementState};
use super::primitives::AssetCatalog;
use super::selection::SelectedEntity;

/// Render the asset browser panel
pub fn asset_browser_ui(
    mut contexts: EguiContexts,
    asset_catalog: Res<AssetCatalog>,
    mut placement_state: ResMut<PlacementState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Assets")
        .default_pos([10.0, 100.0])
        .default_width(150.0)
        .show(ctx, |ui| {
            ui.heading("Primitives");
            ui.separator();

            for primitive in &asset_catalog.primitives {
                if ui.button(&primitive.name).clicked() {
                    start_placement(
                        &mut placement_state,
                        primitive.clone(),
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                    );
                }
            }

            if placement_state.active {
                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, "Placement Mode");
                if let Some(prim) = &placement_state.selected_primitive {
                    ui.label(format!("Placing: {}", prim.name));
                }
                ui.label("Click to place");
                ui.label("ESC to cancel");
            }
        });
}

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
