use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::objects::selection::SelectedEntity;

/// Local UI state for text input buffers
#[derive(Resource, Default)]
pub struct InspectorState {
    // Position buffers
    pub pos_x: String,
    pub pos_y: String,
    pub pos_z: String,
    // Rotation buffers (Euler angles in degrees)
    pub rot_x: String,
    pub rot_y: String,
    pub rot_z: String,
    // Scale buffers
    pub scale_x: String,
    pub scale_y: String,
    pub scale_z: String,
    // Track last selected entity to detect changes
    pub last_entity: Option<Entity>,
}

/// Render the inspector panel with editable numeric fields
pub fn inspector_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedEntity>,
    mut transform_query: Query<&mut Transform>,
    name_query: Query<&Name>,
    mut inspector_state: ResMut<InspectorState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::SidePanel::right("inspector")
        .default_width(280.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();

            if let Some(entity) = selected.entity {
                ui.label("Selected Object");
                ui.add_space(4.0);

                // Show entity name if it has one
                if let Ok(name) = name_query.get(entity) {
                    ui.label(format!("Name: {}", name));
                } else {
                    ui.label(format!("Entity: {:?}", entity));
                }

                ui.separator();

                // Update buffers if selection changed
                if inspector_state.last_entity != Some(entity) {
                    inspector_state.last_entity = Some(entity);
                    if let Ok(transform) = transform_query.get(entity) {
                        update_buffers_from_transform(&mut inspector_state, &transform);
                    }
                }

                // Editable transform fields
                if let Ok(mut transform) = transform_query.get_mut(entity) {
                    ui.label("Transform:");

                    // Position
                    ui.group(|ui| {
                        ui.label("Position:");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.pos_x, 60.0, 0.1) {
                                if let Ok(value) = inspector_state.pos_x.parse::<f32>() {
                                    transform.translation.x = value;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.pos_y, 60.0, 0.1) {
                                if let Ok(value) = inspector_state.pos_y.parse::<f32>() {
                                    transform.translation.y = value;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Z:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.pos_z, 60.0, 0.1) {
                                if let Ok(value) = inspector_state.pos_z.parse::<f32>() {
                                    transform.translation.z = value;
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Rotation (Euler angles in degrees)
                    ui.group(|ui| {
                        ui.label("Rotation (degrees):");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.rot_x, 60.0, 5.0) {
                                if let Ok(value) = inspector_state.rot_x.parse::<f32>() {
                                    let (_, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
                                    transform.rotation = Quat::from_euler(
                                        EulerRot::XYZ,
                                        value.to_radians(),
                                        y,
                                        z,
                                    );
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.rot_y, 60.0, 5.0) {
                                if let Ok(value) = inspector_state.rot_y.parse::<f32>() {
                                    let (x, _, z) = transform.rotation.to_euler(EulerRot::XYZ);
                                    transform.rotation = Quat::from_euler(
                                        EulerRot::XYZ,
                                        x,
                                        value.to_radians(),
                                        z,
                                    );
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Z:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.rot_z, 60.0, 5.0) {
                                if let Ok(value) = inspector_state.rot_z.parse::<f32>() {
                                    let (x, y, _) = transform.rotation.to_euler(EulerRot::XYZ);
                                    transform.rotation = Quat::from_euler(
                                        EulerRot::XYZ,
                                        x,
                                        y,
                                        value.to_radians(),
                                    );
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Scale
                    ui.group(|ui| {
                        ui.label("Scale:");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.scale_x, 60.0, 0.1) {
                                if let Ok(value) = inspector_state.scale_x.parse::<f32>() {
                                    transform.scale.x = value.max(0.01); // Prevent zero/negative scale
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.scale_y, 60.0, 0.1) {
                                if let Ok(value) = inspector_state.scale_y.parse::<f32>() {
                                    transform.scale.y = value.max(0.01);
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Z:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.scale_z, 60.0, 0.1) {
                                if let Ok(value) = inspector_state.scale_z.parse::<f32>() {
                                    transform.scale.z = value.max(0.01);
                                }
                            }
                        });
                    });

                    // Update buffers if transform was modified externally (e.g., by gizmo)
                    if transform.is_changed() && !transform.is_added() {
                        update_buffers_from_transform(&mut inspector_state, &transform);
                    }
                }
            } else {
                inspector_state.last_entity = None;
                ui.label("No selection");
                ui.separator();
                ui.label("Click an object to select");
            }
        });
}

/// Helper function to render an editable float field with validation
/// Returns true if the value was changed (Enter pressed or focus lost)
fn edit_float_field(ui: &mut egui::Ui, buffer: &mut String, width: f32) -> bool {
    let mut response = ui.add(
        egui::TextEdit::singleline(buffer)
            .desired_width(width)
            .char_limit(12),
    );

    // Validate on text change (highlight invalid input)
    let is_valid = buffer.parse::<f32>().is_ok() || buffer.is_empty();

    // Add visual feedback in the same row
    if !is_valid {
        response = response.on_hover_text("Invalid number");
    }

    // Apply changes on Enter key
    let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

    enter_pressed && is_valid
}

/// Helper function to render an editable float field with +/- stepper buttons
/// Returns true if the value was changed
fn edit_float_field_with_steppers(
    ui: &mut egui::Ui,
    buffer: &mut String,
    width: f32,
    step: f32,
) -> bool {
    let mut changed = false;

    // Decrement button
    if ui.small_button("−").clicked() {
        if let Ok(value) = buffer.parse::<f32>() {
            *buffer = format!("{:.3}", value - step);
            changed = true;
        }
    }

    // Text field
    changed |= edit_float_field(ui, buffer, width);

    // Increment button
    if ui.small_button("+").clicked() {
        if let Ok(value) = buffer.parse::<f32>() {
            *buffer = format!("{:.3}", value + step);
            changed = true;
        }
    }

    changed
}

/// Update text buffers from transform (when selection changes or transform modified externally)
fn update_buffers_from_transform(state: &mut InspectorState, transform: &Transform) {
    // Position
    state.pos_x = format!("{:.3}", transform.translation.x);
    state.pos_y = format!("{:.3}", transform.translation.y);
    state.pos_z = format!("{:.3}", transform.translation.z);

    // Rotation (convert to degrees)
    let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
    state.rot_x = format!("{:.3}", x.to_degrees());
    state.rot_y = format!("{:.3}", y.to_degrees());
    state.rot_z = format!("{:.3}", z.to_degrees());

    // Scale
    state.scale_x = format!("{:.3}", transform.scale.x);
    state.scale_y = format!("{:.3}", transform.scale.y);
    state.scale_z = format!("{:.3}", transform.scale.z);
}
