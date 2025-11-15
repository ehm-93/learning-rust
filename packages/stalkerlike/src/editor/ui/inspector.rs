use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::objects::selection::SelectionSet;
use crate::editor::core::types::{RigidBodyType, EditorLight};

/// Local UI state for text input buffers
#[derive(Resource)]
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

    // Lighting buffers
    pub lighting_enabled: bool,
    pub dir_light_illuminance: String,
    pub dir_light_color: [f32; 3],
    pub dir_light_color_hex: String,
    pub dir_light_rotation_x: String, // Pitch (up/down)
    pub dir_light_rotation_y: String, // Yaw (left/right)
    pub ambient_brightness: String,
    pub ambient_color: [f32; 3],
    pub ambient_color_hex: String,

    // Per-entity light buffers
    pub point_light_intensity: String,
    pub point_light_range: String,
    pub point_light_color: [f32; 3],
    pub point_light_color_hex: String,
    pub spot_light_intensity: String,
    pub spot_light_range: String,
    pub spot_light_color: [f32; 3],
    pub spot_light_color_hex: String,
    pub spot_light_inner_angle: String,
    pub spot_light_outer_angle: String,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            pos_x: String::new(),
            pos_y: String::new(),
            pos_z: String::new(),
            rot_x: String::new(),
            rot_y: String::new(),
            rot_z: String::new(),
            scale_x: String::new(),
            scale_y: String::new(),
            scale_z: String::new(),
            last_entity: None,
            lighting_enabled: true, // Start with lighting enabled
            dir_light_illuminance: String::new(),
            dir_light_color: [1.0, 1.0, 1.0],
            dir_light_color_hex: String::from("ffffff"),
            dir_light_rotation_x: String::new(),
            dir_light_rotation_y: String::new(),
            ambient_brightness: String::new(),
            ambient_color: [1.0, 1.0, 1.0],
            ambient_color_hex: String::from("ffffff"),
            point_light_intensity: String::new(),
            point_light_range: String::new(),
            point_light_color: [1.0, 1.0, 0.9],
            point_light_color_hex: String::from("fff4e6"),
            spot_light_intensity: String::new(),
            spot_light_range: String::new(),
            spot_light_color: [1.0, 1.0, 0.9],
            spot_light_color_hex: String::from("fff4e6"),
            spot_light_inner_angle: String::new(),
            spot_light_outer_angle: String::new(),
        }
    }
}

/// Render the inspector panel with editable numeric fields
/// For multi-select, shows aggregate information instead of individual properties
pub fn inspector_ui(
    mut contexts: EguiContexts,
    selection: Res<SelectionSet>,
    mut transform_query: Query<&mut Transform, With<crate::editor::core::types::EditorEntity>>,
    name_query: Query<&Name>,
    mut inspector_state: ResMut<InspectorState>,
    mut rigid_body_query: Query<Option<&mut RigidBodyType>>,
    mut commands: Commands,
    mut directional_light: Query<(&mut DirectionalLight, &mut Transform), Without<crate::editor::core::types::EditorEntity>>,
    mut ambient_light: ResMut<AmbientLight>,
    mut lighting_enabled: ResMut<crate::editor::viewport::LightingEnabled>,
    editor_light_query: Query<&EditorLight>,
    mut point_light_query: Query<&mut PointLight>,
    mut spot_light_query: Query<&mut SpotLight>,
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

            if selection.is_empty() {
                inspector_state.last_entity = None;

                // Show global lighting controls
                ui.label("Global Lighting");
                ui.separator();

                // Lighting mode toggle
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    let mut enabled = lighting_enabled.0;
                    if ui.checkbox(&mut enabled, "Custom Lighting").changed() {
                        lighting_enabled.0 = enabled;
                        inspector_state.lighting_enabled = enabled;
                    }
                });

                ui.add_space(4.0);

                if !lighting_enabled.0 {
                    ui.label(egui::RichText::new("Simple mode: 10000 white ambient, no directional").weak().small());
                } else {
                    ui.label(egui::RichText::new("Custom mode: Configure lights below").weak().small());
                }

                ui.add_space(8.0);

                // Initialize lighting buffers if needed
                if inspector_state.dir_light_illuminance.is_empty() {
                    if let Ok((dir_light, transform)) = directional_light.single() {
                        inspector_state.dir_light_illuminance = format!("{:.0}", dir_light.illuminance);
                        let color = dir_light.color.to_srgba();
                        inspector_state.dir_light_color = [color.red, color.green, color.blue];
                        inspector_state.dir_light_color_hex = rgb_to_hex(&inspector_state.dir_light_color);
                        
                        // Get rotation as euler angles (pitch and yaw)
                        let (pitch, yaw, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
                        inspector_state.dir_light_rotation_x = format!("{:.1}", pitch.to_degrees());
                        inspector_state.dir_light_rotation_y = format!("{:.1}", yaw.to_degrees());
                    }
                    let amb_color = ambient_light.color.to_srgba();
                    inspector_state.ambient_brightness = format!("{:.0}", ambient_light.brightness);
                    inspector_state.ambient_color = [amb_color.red, amb_color.green, amb_color.blue];
                    inspector_state.ambient_color_hex = rgb_to_hex(&inspector_state.ambient_color);
                    inspector_state.lighting_enabled = lighting_enabled.0;
                }

                // Only show detailed controls if custom lighting is enabled
                if lighting_enabled.0 {
                    // Directional Light controls
                    ui.group(|ui| {
                        ui.label("Directional Light:");

                        ui.horizontal(|ui| {
                            ui.label("Illuminance:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.dir_light_illuminance, 60.0, 1000.0) {
                                if let Ok(value) = inspector_state.dir_light_illuminance.parse::<f32>() {
                                    if let Ok((mut dir_light, _)) = directional_light.single_mut() {
                                        dir_light.illuminance = value.max(0.0);
                                    }
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            if ui.color_edit_button_rgb(&mut inspector_state.dir_light_color).changed() {
                                inspector_state.dir_light_color_hex = rgb_to_hex(&inspector_state.dir_light_color);
                                if let Ok((mut dir_light, _)) = directional_light.single_mut() {
                                    dir_light.color = Color::srgb(
                                        inspector_state.dir_light_color[0],
                                        inspector_state.dir_light_color[1],
                                        inspector_state.dir_light_color[2],
                                    );
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Hex:");
                            ui.add(egui::TextEdit::singleline(&mut inspector_state.dir_light_color_hex)
                                .desired_width(80.0)
                                .char_limit(6));

                            if ui.small_button("Apply").clicked() {
                                if let Some(rgb) = hex_to_rgb(&inspector_state.dir_light_color_hex) {
                                    inspector_state.dir_light_color = rgb;
                                    if let Ok((mut dir_light, _)) = directional_light.single_mut() {
                                        dir_light.color = Color::srgb(rgb[0], rgb[1], rgb[2]);
                                    }
                                }
                            }
                        });
                        
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Direction:").small());
                        
                        ui.horizontal(|ui| {
                            ui.label("Pitch:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.dir_light_rotation_x, 50.0, 5.0) {
                                if let Ok(pitch_deg) = inspector_state.dir_light_rotation_x.parse::<f32>() {
                                    if let Ok((_, mut transform)) = directional_light.single_mut() {
                                        // Get current yaw
                                        let (_current_pitch, yaw, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
                                        // Set new rotation with updated pitch
                                        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch_deg.to_radians(), 0.0);
                                    }
                                }
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Yaw:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.dir_light_rotation_y, 50.0, 5.0) {
                                if let Ok(yaw_deg) = inspector_state.dir_light_rotation_y.parse::<f32>() {
                                    if let Ok((_, mut transform)) = directional_light.single_mut() {
                                        // Get current pitch
                                        let (pitch, _current_yaw, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
                                        // Set new rotation with updated yaw
                                        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw_deg.to_radians(), pitch, 0.0);
                                    }
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Ambient Light controls
                    ui.group(|ui| {
                        ui.label("Ambient Light:");

                        ui.horizontal(|ui| {
                            ui.label("Brightness:");
                            if edit_float_field_with_steppers(ui, &mut inspector_state.ambient_brightness, 60.0, 50.0) {
                                if let Ok(value) = inspector_state.ambient_brightness.parse::<f32>() {
                                    ambient_light.brightness = value.max(0.0);
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            if ui.color_edit_button_rgb(&mut inspector_state.ambient_color).changed() {
                                inspector_state.ambient_color_hex = rgb_to_hex(&inspector_state.ambient_color);
                                ambient_light.color = Color::srgb(
                                    inspector_state.ambient_color[0],
                                    inspector_state.ambient_color[1],
                                    inspector_state.ambient_color[2],
                                );
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Hex:");
                            ui.add(egui::TextEdit::singleline(&mut inspector_state.ambient_color_hex)
                                .desired_width(80.0)
                                .char_limit(6));

                            if ui.small_button("Apply").clicked() {
                                if let Some(rgb) = hex_to_rgb(&inspector_state.ambient_color_hex) {
                                    inspector_state.ambient_color = rgb;
                                    ambient_light.color = Color::srgb(rgb[0], rgb[1], rgb[2]);
                                }
                            }
                        });
                    });
                }

                return;
            }

            // Multi-select: show aggregate information
            if selection.len() > 1 {
                inspector_state.last_entity = None;
                ui.label(format!("Multiple Objects Selected ({})", selection.len()));
                ui.separator();

                // Calculate bounding box of all selected entities
                let mut min = Vec3::splat(f32::INFINITY);
                let mut max = Vec3::splat(f32::NEG_INFINITY);
                let mut center = Vec3::ZERO;
                let mut count = 0;

                for entity in &selection.entities {
                    if let Ok(transform) = transform_query.get(*entity) {
                        let pos = transform.translation;
                        min = min.min(pos);
                        max = max.max(pos);
                        center += pos;
                        count += 1;
                    }
                }

                if count > 0 {
                    center /= count as f32;
                    let bounds = max - min;

                    ui.group(|ui| {
                        ui.label("Aggregate Properties:");
                        ui.label(format!("Center: ({:.2}, {:.2}, {:.2})", center.x, center.y, center.z));
                        ui.label(format!("Bounds: ({:.2}, {:.2}, {:.2})", bounds.x, bounds.y, bounds.z));
                    });

                    ui.add_space(8.0);
                    ui.label("Tip: Use gizmo to transform all selected objects together");
                }

                return;
            }

            // Single selection: show detailed properties
            let entity = selection.first().unwrap();
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

            ui.add_space(8.0);
            ui.separator();

            // Rigid Body Type selector
            ui.group(|ui| {
                ui.label("Physics:");

                if let Ok(rigid_body_opt) = rigid_body_query.get_mut(entity) {
                    let current_type = rigid_body_opt.map(|rb| *rb).unwrap_or_default();

                    ui.horizontal(|ui| {
                        ui.label("Rigid Body:");

                        egui::ComboBox::from_id_salt("rigid_body_selector")
                            .selected_text(current_type.display_name())
                            .show_ui(ui, |ui| {
                                for &variant in RigidBodyType::variants() {
                                    let response = ui.selectable_label(
                                        current_type == variant,
                                        variant.display_name()
                                    );

                                    if response.clicked() && current_type != variant {
                                        // Add or update the component
                                        commands.entity(entity).insert(variant);
                                    }
                                }
                            });
                    });

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(match current_type {
                        RigidBodyType::Fixed => "Static - does not move",
                        RigidBodyType::Dynamic => "Dynamic - affected by physics",
                    }).weak().small());
                }
            });

            // Light properties (if entity has a light component)
            if let Ok(editor_light) = editor_light_query.get(entity) {
                ui.add_space(8.0);
                ui.separator();

                ui.group(|ui| {
                    ui.label(format!("Light: {}", editor_light.light_type.display_name()));

                    match editor_light.light_type {
                        crate::editor::core::types::LightType::Point => {
                            if let Ok(mut light) = point_light_query.get_mut(entity) {
                                // Update buffers if entity changed
                                if inspector_state.last_entity != Some(entity) {
                                    inspector_state.point_light_intensity = format!("{:.1}", light.intensity);
                                    inspector_state.point_light_range = format!("{:.1}", light.range);
                                    let color = light.color.to_srgba();
                                    inspector_state.point_light_color = [color.red, color.green, color.blue];
                                    inspector_state.point_light_color_hex = format!(
                                        "{:02x}{:02x}{:02x}",
                                        (color.red * 255.0) as u8,
                                        (color.green * 255.0) as u8,
                                        (color.blue * 255.0) as u8
                                    );
                                }

                                ui.horizontal(|ui| {
                                    ui.label("Intensity:");
                                    if edit_float_field_with_steppers(ui, &mut inspector_state.point_light_intensity, 60.0, 100.0) {
                                        if let Ok(value) = inspector_state.point_light_intensity.parse::<f32>() {
                                            light.intensity = value.max(0.0);
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    if ui.color_edit_button_rgb(&mut inspector_state.point_light_color).changed() {
                                        light.color = Color::srgb(
                                            inspector_state.point_light_color[0],
                                            inspector_state.point_light_color[1],
                                            inspector_state.point_light_color[2],
                                        );
                                        // Update hex representation
                                        inspector_state.point_light_color_hex = format!(
                                            "{:02x}{:02x}{:02x}",
                                            (inspector_state.point_light_color[0] * 255.0) as u8,
                                            (inspector_state.point_light_color[1] * 255.0) as u8,
                                            (inspector_state.point_light_color[2] * 255.0) as u8
                                        );
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Hex:");
                                    ui.label("#");
                                    let response = ui.add(egui::TextEdit::singleline(&mut inspector_state.point_light_color_hex).desired_width(60.0));
                                    if response.lost_focus() || response.changed() {
                                        // Parse hex color (with or without #)
                                        let hex = inspector_state.point_light_color_hex.trim_start_matches('#');
                                        if hex.len() == 6 {
                                            if let (Ok(r), Ok(g), Ok(b)) = (
                                                u8::from_str_radix(&hex[0..2], 16),
                                                u8::from_str_radix(&hex[2..4], 16),
                                                u8::from_str_radix(&hex[4..6], 16),
                                            ) {
                                                inspector_state.point_light_color[0] = r as f32 / 255.0;
                                                inspector_state.point_light_color[1] = g as f32 / 255.0;
                                                inspector_state.point_light_color[2] = b as f32 / 255.0;
                                                light.color = Color::srgb(
                                                    inspector_state.point_light_color[0],
                                                    inspector_state.point_light_color[1],
                                                    inspector_state.point_light_color[2],
                                                );
                                            }
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Range:");
                                    if edit_float_field_with_steppers(ui, &mut inspector_state.point_light_range, 60.0, 1.0) {
                                        if let Ok(value) = inspector_state.point_light_range.parse::<f32>() {
                                            light.range = value.max(0.1);
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Shadows:");
                                    ui.checkbox(&mut light.shadows_enabled, "");
                                });
                            }
                        }
                        crate::editor::core::types::LightType::Spot => {
                            if let Ok(mut light) = spot_light_query.get_mut(entity) {
                                // Update buffers if entity changed
                                if inspector_state.last_entity != Some(entity) {
                                    inspector_state.spot_light_intensity = format!("{:.1}", light.intensity);
                                    inspector_state.spot_light_range = format!("{:.1}", light.range);
                                    inspector_state.spot_light_inner_angle = format!("{:.3}", light.inner_angle.to_degrees());
                                    inspector_state.spot_light_outer_angle = format!("{:.3}", light.outer_angle.to_degrees());
                                    let color = light.color.to_srgba();
                                    inspector_state.spot_light_color = [color.red, color.green, color.blue];
                                    inspector_state.spot_light_color_hex = format!(
                                        "{:02x}{:02x}{:02x}",
                                        (color.red * 255.0) as u8,
                                        (color.green * 255.0) as u8,
                                        (color.blue * 255.0) as u8
                                    );
                                }

                                ui.horizontal(|ui| {
                                    ui.label("Intensity:");
                                    if edit_float_field_with_steppers(ui, &mut inspector_state.spot_light_intensity, 60.0, 100.0) {
                                        if let Ok(value) = inspector_state.spot_light_intensity.parse::<f32>() {
                                            light.intensity = value.max(0.0);
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    if ui.color_edit_button_rgb(&mut inspector_state.spot_light_color).changed() {
                                        light.color = Color::srgb(
                                            inspector_state.spot_light_color[0],
                                            inspector_state.spot_light_color[1],
                                            inspector_state.spot_light_color[2],
                                        );
                                        // Update hex representation
                                        inspector_state.spot_light_color_hex = format!(
                                            "{:02x}{:02x}{:02x}",
                                            (inspector_state.spot_light_color[0] * 255.0) as u8,
                                            (inspector_state.spot_light_color[1] * 255.0) as u8,
                                            (inspector_state.spot_light_color[2] * 255.0) as u8
                                        );
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Hex:");
                                    ui.label("#");
                                    let response = ui.add(egui::TextEdit::singleline(&mut inspector_state.spot_light_color_hex).desired_width(60.0));
                                    if response.lost_focus() || response.changed() {
                                        // Parse hex color (with or without #)
                                        let hex = inspector_state.spot_light_color_hex.trim_start_matches('#');
                                        if hex.len() == 6 {
                                            if let (Ok(r), Ok(g), Ok(b)) = (
                                                u8::from_str_radix(&hex[0..2], 16),
                                                u8::from_str_radix(&hex[2..4], 16),
                                                u8::from_str_radix(&hex[4..6], 16),
                                            ) {
                                                inspector_state.spot_light_color[0] = r as f32 / 255.0;
                                                inspector_state.spot_light_color[1] = g as f32 / 255.0;
                                                inspector_state.spot_light_color[2] = b as f32 / 255.0;
                                                light.color = Color::srgb(
                                                    inspector_state.spot_light_color[0],
                                                    inspector_state.spot_light_color[1],
                                                    inspector_state.spot_light_color[2],
                                                );
                                            }
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Range:");
                                    if edit_float_field_with_steppers(ui, &mut inspector_state.spot_light_range, 60.0, 1.0) {
                                        if let Ok(value) = inspector_state.spot_light_range.parse::<f32>() {
                                            light.range = value.max(0.1);
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Inner Angle:");
                                    if edit_float_field_with_steppers(ui, &mut inspector_state.spot_light_inner_angle, 60.0, 5.0) {
                                        if let Ok(value) = inspector_state.spot_light_inner_angle.parse::<f32>() {
                                            light.inner_angle = value.to_radians().max(0.0);
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Outer Angle:");
                                    if edit_float_field_with_steppers(ui, &mut inspector_state.spot_light_outer_angle, 60.0, 5.0) {
                                        if let Ok(value) = inspector_state.spot_light_outer_angle.parse::<f32>() {
                                            light.outer_angle = value.to_radians().max(0.0);
                                        }
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Shadows:");
                                    ui.checkbox(&mut light.shadows_enabled, "");
                                });
                            }
                        }
                    }
                });
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

/// Convert RGB [0-1] to hex string
fn rgb_to_hex(rgb: &[f32; 3]) -> String {
    format!("{:02x}{:02x}{:02x}",
        (rgb[0] * 255.0).round() as u8,
        (rgb[1] * 255.0).round() as u8,
        (rgb[2] * 255.0).round() as u8)
}

/// Parse hex string to RGB [0-1], returns None if invalid
fn hex_to_rgb(hex: &str) -> Option<[f32; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0])
}
