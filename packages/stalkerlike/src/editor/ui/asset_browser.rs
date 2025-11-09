use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::objects::{placement::{start_placement, PlacementState}, primitives::AssetCatalog};

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
