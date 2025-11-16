use bevy::prelude::*;
use bevy::ecs::component::Mutable;
use bevy_egui::{egui, EguiContexts};
use std::any::TypeId;
use std::collections::HashMap;

use crate::editor::objects::selection::SelectionSet;
use crate::editor::core::types::{RigidBodyType, EditorLight, LightType, EditorEntity};
use crate::editor::viewport::LightingEnabled;

// ============================================================================
// Core Property System
// ============================================================================

/// A type-erased property value that can be edited in the inspector
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Float(f32),
    Vec3(Vec3),
    Quat(Quat),  // For rotation
    Color(Color),
    Bool(bool),
    String(String),
    Enum(String, Vec<String>), // (current, options)
}

/// Represents a single editable property
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
    pub metadata: PropertyMetadata,
}

/// Metadata for controlling how a property is displayed/edited
#[derive(Clone, Default)]
pub struct PropertyMetadata {
    pub step: Option<f32>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub tooltip: Option<String>,
    pub read_only: bool,
}

// ============================================================================
// Inspector Traits
// ============================================================================

/// Trait for components that can be inspected
pub trait Inspectable: Component {
    /// Get all properties for this component
    fn properties(&self) -> Vec<Property>;

    /// Apply a property change
    fn set_property(&mut self, name: &str, value: PropertyValue);

    /// Get display name for this component
    fn display_name() -> &'static str where Self: Sized;
}

/// Trait for custom property editors
pub trait PropertyEditor: Send + Sync {
    /// Render the property editor UI
    /// Returns true if the value was modified
    fn render(&mut self, ui: &mut egui::Ui, property: &mut Property) -> bool;
}

// ============================================================================
// Built-in Property Editors
// ============================================================================

pub struct FloatEditor {
    buffer: String,
    last_value: Option<f32>,
}

impl Default for FloatEditor {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            last_value: None,
        }
    }
}

impl PropertyEditor for FloatEditor {
    fn render(&mut self, ui: &mut egui::Ui, property: &mut Property) -> bool {
        let PropertyValue::Float(ref mut value) = property.value else {
            return false;
        };

        // Update buffer if value changed externally
        if self.last_value != Some(*value) {
            self.buffer = format!("{:.3}", value);
            self.last_value = Some(*value);
        }

        let mut changed = false;
        let step = property.metadata.step.unwrap_or(0.1);

        ui.horizontal(|ui| {
            ui.label(&property.name);

            if !property.metadata.read_only {
                if ui.small_button("−").clicked() {
                    *value -= step;
                    if let Some(min) = property.metadata.min {
                        *value = value.max(min);
                    }
                    self.buffer = format!("{:.3}", value);
                    changed = true;
                }

                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.buffer)
                        .desired_width(60.0)
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(new_value) = self.buffer.parse::<f32>() {
                        let mut clamped = new_value;
                        if let Some(min) = property.metadata.min {
                            clamped = clamped.max(min);
                        }
                        if let Some(max) = property.metadata.max {
                            clamped = clamped.min(max);
                        }
                        *value = clamped;
                        self.buffer = format!("{:.3}", value);
                        changed = true;
                    }
                }

                if ui.small_button("+").clicked() {
                    *value += step;
                    if let Some(max) = property.metadata.max {
                        *value = value.min(max);
                    }
                    self.buffer = format!("{:.3}", value);
                    changed = true;
                }
            } else {
                ui.label(format!("{:.3}", value));
            }

            if let Some(tooltip) = &property.metadata.tooltip {
                ui.label("ⓘ").on_hover_text(tooltip);
            }
        });

        changed
    }
}

pub struct Vec3Editor {
    x_editor: FloatEditor,
    y_editor: FloatEditor,
    z_editor: FloatEditor,
}

impl Default for Vec3Editor {
    fn default() -> Self {
        Self {
            x_editor: FloatEditor::default(),
            y_editor: FloatEditor::default(),
            z_editor: FloatEditor::default(),
        }
    }
}

impl PropertyEditor for Vec3Editor {
    fn render(&mut self, ui: &mut egui::Ui, property: &mut Property) -> bool {
        let PropertyValue::Vec3(ref mut value) = property.value else {
            return false;
        };

        let mut changed = false;

        ui.group(|ui| {
            ui.label(&property.name);

            let mut x_prop = Property {
                name: "X".to_string(),
                value: PropertyValue::Float(value.x),
                metadata: property.metadata.clone(),
            };
            changed |= self.x_editor.render(ui, &mut x_prop);
            if let PropertyValue::Float(x) = x_prop.value {
                value.x = x;
            }

            let mut y_prop = Property {
                name: "Y".to_string(),
                value: PropertyValue::Float(value.y),
                metadata: property.metadata.clone(),
            };
            changed |= self.y_editor.render(ui, &mut y_prop);
            if let PropertyValue::Float(y) = y_prop.value {
                value.y = y;
            }

            let mut z_prop = Property {
                name: "Z".to_string(),
                value: PropertyValue::Float(value.z),
                metadata: property.metadata.clone(),
            };
            changed |= self.z_editor.render(ui, &mut z_prop);
            if let PropertyValue::Float(z) = z_prop.value {
                value.z = z;
            }
        });

        changed
    }
}

pub struct QuatEditor {
    x_editor: FloatEditor,
    y_editor: FloatEditor,
    z_editor: FloatEditor,
}

impl Default for QuatEditor {
    fn default() -> Self {
        Self {
            x_editor: FloatEditor::default(),
            y_editor: FloatEditor::default(),
            z_editor: FloatEditor::default(),
        }
    }
}

impl PropertyEditor for QuatEditor {
    fn render(&mut self, ui: &mut egui::Ui, property: &mut Property) -> bool {
        let PropertyValue::Quat(ref mut quat) = property.value else {
            return false;
        };

        let mut changed = false;

        ui.group(|ui| {
            ui.label(&property.name);

            // Convert to Euler angles for editing
            let (x, y, z) = quat.to_euler(EulerRot::XYZ);
            let mut euler_degrees = Vec3::new(
                x.to_degrees(),
                y.to_degrees(),
                z.to_degrees()
            );

            // X rotation
            ui.horizontal(|ui| {
                ui.label("X:");
                let mut x_prop = Property {
                    name: String::new(),
                    value: PropertyValue::Float(euler_degrees.x),
                    metadata: PropertyMetadata {
                        step: Some(5.0),
                        ..property.metadata.clone()
                    },
                };
                if self.x_editor.render(ui, &mut x_prop) {
                    if let PropertyValue::Float(deg) = x_prop.value {
                        euler_degrees.x = deg;
                        changed = true;
                    }
                }
            });

            // Y rotation
            ui.horizontal(|ui| {
                ui.label("Y:");
                let mut y_prop = Property {
                    name: String::new(),
                    value: PropertyValue::Float(euler_degrees.y),
                    metadata: PropertyMetadata {
                        step: Some(5.0),
                        ..property.metadata.clone()
                    },
                };
                if self.y_editor.render(ui, &mut y_prop) {
                    if let PropertyValue::Float(deg) = y_prop.value {
                        euler_degrees.y = deg;
                        changed = true;
                    }
                }
            });

            // Z rotation
            ui.horizontal(|ui| {
                ui.label("Z:");
                let mut z_prop = Property {
                    name: String::new(),
                    value: PropertyValue::Float(euler_degrees.z),
                    metadata: PropertyMetadata {
                        step: Some(5.0),
                        ..property.metadata.clone()
                    },
                };
                if self.z_editor.render(ui, &mut z_prop) {
                    if let PropertyValue::Float(deg) = z_prop.value {
                        euler_degrees.z = deg;
                        changed = true;
                    }
                }
            });

            if changed {
                *quat = Quat::from_euler(
                    EulerRot::XYZ,
                    euler_degrees.x.to_radians(),
                    euler_degrees.y.to_radians(),
                    euler_degrees.z.to_radians()
                );
            }
        });

        changed
    }
}

pub struct ColorEditor {
    hex_buffer: String,
    last_color: Option<Color>,
}

impl Default for ColorEditor {
    fn default() -> Self {
        Self {
            hex_buffer: String::from("ffffff"),
            last_color: None,
        }
    }
}

impl PropertyEditor for ColorEditor {
    fn render(&mut self, ui: &mut egui::Ui, property: &mut Property) -> bool {
        let PropertyValue::Color(ref mut color) = property.value else {
            return false;
        };

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label(&property.name);

            let srgba = color.to_srgba();
            let mut rgb = [srgba.red, srgba.green, srgba.blue];

            if ui.color_edit_button_rgb(&mut rgb).changed() {
                *color = Color::srgb(rgb[0], rgb[1], rgb[2]);
                self.hex_buffer = format!("{:02x}{:02x}{:02x}",
                    (rgb[0] * 255.0) as u8,
                    (rgb[1] * 255.0) as u8,
                    (rgb[2] * 255.0) as u8
                );
                changed = true;
            }

            ui.label("#");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.hex_buffer)
                    .desired_width(60.0)
                    .char_limit(6)
            );

            if response.lost_focus() || response.changed() {
                if let Some(parsed_color) = parse_hex_color(&self.hex_buffer) {
                    *color = parsed_color;
                    changed = true;
                }
            }
        });

        changed
    }
}

pub struct BoolEditor;

impl PropertyEditor for BoolEditor {
    fn render(&mut self, ui: &mut egui::Ui, property: &mut Property) -> bool {
        let PropertyValue::Bool(ref mut value) = property.value else {
            return false;
        };

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label(&property.name);
            if ui.checkbox(value, "").changed() {
                changed = true;
            }

            if let Some(tooltip) = &property.metadata.tooltip {
                ui.label("ⓘ").on_hover_text(tooltip);
            }
        });

        changed
    }
}

pub struct EnumEditor;

impl PropertyEditor for EnumEditor {
    fn render(&mut self, ui: &mut egui::Ui, property: &mut Property) -> bool {
        let PropertyValue::Enum(ref mut current, ref options) = property.value else {
            return false;
        };

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label(&property.name);

            egui::ComboBox::from_id_salt(&property.name)
                .selected_text(current.as_str())
                .show_ui(ui, |ui| {
                    for option in options {
                        if ui.selectable_label(current == option, option).clicked() {
                            *current = option.clone();
                            changed = true;
                        }
                    }
                });
        });

        changed
    }
}

// ============================================================================
// Inspector Registry
// ============================================================================

/// Registry for component inspectors
#[derive(Resource, Default)]
pub struct InspectorRegistry {
    /// Maps TypeId to inspector functions
    inspectors: HashMap<TypeId, Box<dyn ComponentInspector>>,
}

impl InspectorRegistry {
    /// Register an inspector for a component type
    pub fn register<T>(&mut self)
    where
        T: Inspectable + Component<Mutability = Mutable> + 'static,
    {
        self.inspectors.insert(
            TypeId::of::<T>(),
            Box::new(InspectableAdapter::<T>::default()),
        );
    }

    /// Get inspector for a type
    pub fn get(&self, type_id: TypeId) -> Option<&dyn ComponentInspector> {
        self.inspectors.get(&type_id).map(|b| b.as_ref())
    }
}

/// Trait for component inspection
pub trait ComponentInspector: Send + Sync {
    /// Get properties from entity
    fn get_properties(&self, world: &World, entity: Entity) -> Option<Vec<Property>>;

    /// Set property on entity
    fn set_property(
        &self,
        world: &mut World,
        entity: Entity,
        name: &str,
        value: PropertyValue,
    );

    /// Get display name
    fn display_name(&self) -> &'static str;
}

/// Adapter to make Inspectable components work with the registry
struct InspectableAdapter<T: Inspectable> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Inspectable> Default for InspectableAdapter<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> ComponentInspector for InspectableAdapter<T>
where
    T: Inspectable + Component<Mutability = Mutable> + 'static,
{
    fn get_properties(&self, world: &World, entity: Entity) -> Option<Vec<Property>> {
        world.get::<T>(entity).map(|component| component.properties())
    }

    fn set_property(
        &self,
        world: &mut World,
        entity: Entity,
        name: &str,
        value: PropertyValue,
    ) {
        if let Some(mut component) = world.get_mut::<T>(entity) {
            component.set_property(name, value);
        }
    }

    fn display_name(&self) -> &'static str {
        T::display_name()
    }
}

// ============================================================================
// Inspector State
// ============================================================================

/// State for the inspector panel
#[derive(Resource)]
pub struct InspectorState {
    /// Currently selected entity
    selected_entity: Option<Entity>,
    /// Property editors for each component
    editors: HashMap<TypeId, HashMap<String, Box<dyn PropertyEditor>>>,
    /// Global lighting state (for when nothing is selected)
    lighting_editors: HashMap<String, Box<dyn PropertyEditor>>,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            selected_entity: None,
            editors: HashMap::new(),
            lighting_editors: HashMap::new(),
        }
    }
}

impl InspectorState {
    /// Get or create an editor for a property
    fn get_or_create_editor(
        &mut self,
        component_type: TypeId,
        property_name: &str,
        property_value: &PropertyValue,
    ) -> &mut dyn PropertyEditor {
        let component_editors = self.editors.entry(component_type).or_default();

        component_editors
            .entry(property_name.to_string())
            .or_insert_with(|| create_editor_for_value(property_value))
            .as_mut()
    }

    fn get_or_create_lighting_editor(
        &mut self,
        property_name: &str,
        property_value: &PropertyValue,
    ) -> &mut dyn PropertyEditor {
        self.lighting_editors
            .entry(property_name.to_string())
            .or_insert_with(|| create_editor_for_value(property_value))
            .as_mut()
    }
}

fn create_editor_for_value(value: &PropertyValue) -> Box<dyn PropertyEditor> {
    match value {
        PropertyValue::Float(_) => Box::new(FloatEditor::default()),
        PropertyValue::Vec3(_) => Box::new(Vec3Editor::default()),
        PropertyValue::Quat(_) => Box::new(QuatEditor::default()),
        PropertyValue::Color(_) => Box::new(ColorEditor::default()),
        PropertyValue::Bool(_) => Box::new(BoolEditor),
        PropertyValue::Enum(_, _) => Box::new(EnumEditor),
        PropertyValue::String(_) => Box::new(FloatEditor::default()), // Fallback
    }
}

// ============================================================================
// Component Implementations
// ============================================================================

impl Inspectable for Transform {
    fn properties(&self) -> Vec<Property> {
        vec![
            Property {
                name: "Position".to_string(),
                value: PropertyValue::Vec3(self.translation),
                metadata: PropertyMetadata {
                    step: Some(0.1),
                    ..default()
                },
            },
            Property {
                name: "Rotation".to_string(),
                value: PropertyValue::Quat(self.rotation),
                metadata: PropertyMetadata::default(),
            },
            Property {
                name: "Scale".to_string(),
                value: PropertyValue::Vec3(self.scale),
                metadata: PropertyMetadata {
                    step: Some(0.1),
                    min: Some(0.01),
                    ..default()
                },
            },
        ]
    }

    fn set_property(&mut self, name: &str, value: PropertyValue) {
        match name {
            "Position" => {
                if let PropertyValue::Vec3(v) = value {
                    self.translation = v;
                }
            }
            "Rotation" => {
                if let PropertyValue::Quat(q) = value {
                    self.rotation = q;
                }
            }
            "Scale" => {
                if let PropertyValue::Vec3(v) = value {
                    self.scale = v;
                }
            }
            _ => {}
        }
    }

    fn display_name() -> &'static str {
        "Transform"
    }
}

impl Inspectable for RigidBodyType {
    fn properties(&self) -> Vec<Property> {
        vec![
            Property {
                name: "Type".to_string(),
                value: PropertyValue::Enum(
                    self.display_name().to_string(),
                    RigidBodyType::variants()
                        .iter()
                        .map(|v| v.display_name().to_string())
                        .collect(),
                ),
                metadata: PropertyMetadata::default(),
            },
        ]
    }

    fn set_property(&mut self, name: &str, value: PropertyValue) {
        if name == "Type" {
            if let PropertyValue::Enum(selected, _) = value {
                for variant in RigidBodyType::variants() {
                    if variant.display_name() == selected {
                        *self = *variant;
                        break;
                    }
                }
            }
        }
    }

    fn display_name() -> &'static str {
        "Rigid Body"
    }
}

impl Inspectable for PointLight {
    fn properties(&self) -> Vec<Property> {
        vec![
            Property {
                name: "Intensity".to_string(),
                value: PropertyValue::Float(self.intensity),
                metadata: PropertyMetadata {
                    step: Some(100.0),
                    min: Some(0.0),
                    tooltip: Some("Light intensity in lumens".to_string()),
                    ..default()
                },
            },
            Property {
                name: "Range".to_string(),
                value: PropertyValue::Float(self.range),
                metadata: PropertyMetadata {
                    step: Some(1.0),
                    min: Some(0.1),
                    tooltip: Some("Maximum distance the light affects".to_string()),
                    ..default()
                },
            },
            Property {
                name: "Color".to_string(),
                value: PropertyValue::Color(self.color),
                metadata: PropertyMetadata::default(),
            },
            Property {
                name: "Shadows".to_string(),
                value: PropertyValue::Bool(self.shadows_enabled),
                metadata: PropertyMetadata::default(),
            },
        ]
    }

    fn set_property(&mut self, name: &str, value: PropertyValue) {
        match name {
            "Intensity" => {
                if let PropertyValue::Float(v) = value {
                    self.intensity = v;
                }
            }
            "Range" => {
                if let PropertyValue::Float(v) = value {
                    self.range = v;
                }
            }
            "Color" => {
                if let PropertyValue::Color(v) = value {
                    self.color = v;
                }
            }
            "Shadows" => {
                if let PropertyValue::Bool(v) = value {
                    self.shadows_enabled = v;
                }
            }
            _ => {}
        }
    }

    fn display_name() -> &'static str {
        "Point Light"
    }
}

impl Inspectable for SpotLight {
    fn properties(&self) -> Vec<Property> {
        vec![
            Property {
                name: "Intensity".to_string(),
                value: PropertyValue::Float(self.intensity),
                metadata: PropertyMetadata {
                    step: Some(100.0),
                    min: Some(0.0),
                    ..default()
                },
            },
            Property {
                name: "Range".to_string(),
                value: PropertyValue::Float(self.range),
                metadata: PropertyMetadata {
                    step: Some(1.0),
                    min: Some(0.1),
                    ..default()
                },
            },
            Property {
                name: "Inner Angle".to_string(),
                value: PropertyValue::Float(self.inner_angle.to_degrees()),
                metadata: PropertyMetadata {
                    step: Some(5.0),
                    min: Some(0.0),
                    max: Some(180.0),
                    tooltip: Some("Inner cone angle in degrees".to_string()),
                    ..default()
                },
            },
            Property {
                name: "Outer Angle".to_string(),
                value: PropertyValue::Float(self.outer_angle.to_degrees()),
                metadata: PropertyMetadata {
                    step: Some(5.0),
                    min: Some(0.0),
                    max: Some(180.0),
                    tooltip: Some("Outer cone angle in degrees".to_string()),
                    ..default()
                },
            },
            Property {
                name: "Color".to_string(),
                value: PropertyValue::Color(self.color),
                metadata: PropertyMetadata::default(),
            },
            Property {
                name: "Shadows".to_string(),
                value: PropertyValue::Bool(self.shadows_enabled),
                metadata: PropertyMetadata::default(),
            },
        ]
    }

    fn set_property(&mut self, name: &str, value: PropertyValue) {
        match name {
            "Intensity" => {
                if let PropertyValue::Float(v) = value {
                    self.intensity = v;
                }
            }
            "Range" => {
                if let PropertyValue::Float(v) = value {
                    self.range = v;
                }
            }
            "Inner Angle" => {
                if let PropertyValue::Float(degrees) = value {
                    self.inner_angle = degrees.to_radians();
                }
            }
            "Outer Angle" => {
                if let PropertyValue::Float(degrees) = value {
                    self.outer_angle = degrees.to_radians();
                }
            }
            "Color" => {
                if let PropertyValue::Color(v) = value {
                    self.color = v;
                }
            }
            "Shadows" => {
                if let PropertyValue::Bool(v) = value {
                    self.shadows_enabled = v;
                }
            }
            _ => {}
        }
    }

    fn display_name() -> &'static str {
        "Spot Light"
    }
}

// ============================================================================
// Inspector UI System
// ============================================================================

pub fn inspector_ui(
    mut contexts: EguiContexts,
    selection: Res<SelectionSet>,
    mut state: ResMut<InspectorState>,
    _registry: Res<InspectorRegistry>,
    mut transform_query: Query<&mut Transform, With<EditorEntity>>,
    mut rigid_body_query: Query<&mut RigidBodyType>,
    mut point_light_query: Query<&mut PointLight>,
    mut spot_light_query: Query<&mut SpotLight>,
    editor_light_query: Query<&EditorLight>,
    name_query: Query<&Name>,
    mut commands: Commands,
    mut directional_light: Query<(&mut DirectionalLight, &mut Transform), Without<EditorEntity>>,
    mut ambient_light: ResMut<AmbientLight>,
    mut lighting_enabled: ResMut<LightingEnabled>,
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

            // Handle no selection - show global lighting controls
            if selection.is_empty() {
                state.selected_entity = None;
                show_global_lighting_controls(ui, &mut state, &mut directional_light, &mut ambient_light, &mut lighting_enabled);
                return;
            }

            // Handle multi-selection
            if selection.len() > 1 {
                state.selected_entity = None;
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

            // Single selection
            let entity = selection.first().unwrap();

            // Update selected entity
            if state.selected_entity != Some(entity) {
                state.selected_entity = Some(entity);
                // Clear editors when selection changes
                state.editors.clear();
            }

            // Display entity info
            if let Ok(name) = name_query.get(entity) {
                ui.label(format!("Name: {}", name));
            } else {
                ui.label(format!("Entity: {:?}", entity));
            }
            ui.separator();

            // Transform component (always present)
            if let Ok(mut transform) = transform_query.get_mut(entity) {
                let mut properties = transform.properties();
                let mut changed_properties = Vec::new();

                ui.collapsing("Transform", |ui| {
                    for mut property in &mut properties {
                        let editor = state.get_or_create_editor(
                            TypeId::of::<Transform>(),
                            &property.name,
                            &property.value,
                        );

                        if editor.render(ui, &mut property) {
                            changed_properties.push((property.name.clone(), property.value.clone()));
                        }
                    }
                });

                // Apply changes
                for (name, value) in changed_properties {
                    transform.set_property(&name, value);
                }
            }

            ui.add_space(8.0);

            // RigidBody component
            if let Ok(rigid_body) = rigid_body_query.get_mut(entity) {
                let mut properties = rigid_body.properties();

                ui.collapsing("Physics", |ui| {
                    for mut property in &mut properties {
                        let editor = state.get_or_create_editor(
                            TypeId::of::<RigidBodyType>(),
                            &property.name,
                            &property.value,
                        );

                        if editor.render(ui, &mut property) {
                            // For enums, we need to handle component replacement
                            if let PropertyValue::Enum(ref selected, _) = property.value {
                                for variant in RigidBodyType::variants() {
                                    if variant.display_name() == selected {
                                        commands.entity(entity).insert(*variant);
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(match *rigid_body {
                        RigidBodyType::Fixed => "Static - does not move",
                        RigidBodyType::Dynamic => "Dynamic - affected by physics",
                    }).weak().small());
                });
            }

            // Light components
            if let Ok(editor_light) = editor_light_query.get(entity) {
                ui.add_space(8.0);

                match editor_light.light_type {
                    LightType::Point => {
                        if let Ok(mut light) = point_light_query.get_mut(entity) {
                            let mut properties = light.properties();
                            let mut changed_properties = Vec::new();

                            ui.collapsing("Point Light", |ui| {
                                for mut property in &mut properties {
                                    let editor = state.get_or_create_editor(
                                        TypeId::of::<PointLight>(),
                                        &property.name,
                                        &property.value,
                                    );

                                    if editor.render(ui, &mut property) {
                                        changed_properties.push((property.name.clone(), property.value.clone()));
                                    }
                                }
                            });

                            for (name, value) in changed_properties {
                                light.set_property(&name, value);
                            }
                        }
                    }
                    LightType::Spot => {
                        if let Ok(mut light) = spot_light_query.get_mut(entity) {
                            let mut properties = light.properties();
                            let mut changed_properties = Vec::new();

                            ui.collapsing("Spot Light", |ui| {
                                for mut property in &mut properties {
                                    let editor = state.get_or_create_editor(
                                        TypeId::of::<SpotLight>(),
                                        &property.name,
                                        &property.value,
                                    );

                                    if editor.render(ui, &mut property) {
                                        changed_properties.push((property.name.clone(), property.value.clone()));
                                    }
                                }
                            });

                            for (name, value) in changed_properties {
                                light.set_property(&name, value);
                            }
                        }
                    }
                }
            }
        });
}

// Global lighting controls when nothing is selected
fn show_global_lighting_controls(
    ui: &mut egui::Ui,
    state: &mut InspectorState,
    directional_light: &mut Query<(&mut DirectionalLight, &mut Transform), Without<EditorEntity>>,
    ambient_light: &mut ResMut<AmbientLight>,
    lighting_enabled: &mut ResMut<LightingEnabled>,
) {
    ui.label("Global Lighting");
    ui.separator();

    // Lighting mode toggle
    ui.horizontal(|ui| {
        ui.label("Mode:");
        if ui.checkbox(&mut lighting_enabled.0, "Custom Lighting").changed() {
            // Mode changed
        }
    });

    ui.add_space(4.0);

    if !lighting_enabled.0 {
        ui.label(egui::RichText::new("Simple mode: 10000 white ambient, no directional").weak().small());
    } else {
        ui.label(egui::RichText::new("Custom mode: Configure lights below").weak().small());
    }

    ui.add_space(8.0);

    if !lighting_enabled.0 {
        return;
    }

    // Directional Light controls
    if let Ok((mut dir_light, mut transform)) = directional_light.single_mut() {
        ui.group(|ui| {
            ui.label("Directional Light:");

            // Illuminance
            let mut illum_prop = Property {
                name: "Illuminance".to_string(),
                value: PropertyValue::Float(dir_light.illuminance),
                metadata: PropertyMetadata {
                    step: Some(1000.0),
                    min: Some(0.0),
                    ..default()
                },
            };

            let editor = state.get_or_create_lighting_editor("dir_illuminance", &illum_prop.value);
            if editor.render(ui, &mut illum_prop) {
                if let PropertyValue::Float(v) = illum_prop.value {
                    dir_light.illuminance = v;
                }
            }

            // Color
            let mut color_prop = Property {
                name: "Color".to_string(),
                value: PropertyValue::Color(dir_light.color),
                metadata: PropertyMetadata::default(),
            };

            let editor = state.get_or_create_lighting_editor("dir_color", &color_prop.value);
            if editor.render(ui, &mut color_prop) {
                if let PropertyValue::Color(c) = color_prop.value {
                    dir_light.color = c;
                }
            }

            // Direction (as euler angles)
            ui.label(egui::RichText::new("Direction:").small());

            let (pitch, yaw, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let mut pitch_deg = pitch.to_degrees();
            let mut yaw_deg = yaw.to_degrees();

            // Pitch
            let mut pitch_prop = Property {
                name: "Pitch".to_string(),
                value: PropertyValue::Float(pitch_deg),
                metadata: PropertyMetadata {
                    step: Some(5.0),
                    ..default()
                },
            };

            let editor = state.get_or_create_lighting_editor("dir_pitch", &pitch_prop.value);
            if editor.render(ui, &mut pitch_prop) {
                if let PropertyValue::Float(v) = pitch_prop.value {
                    pitch_deg = v;
                    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw_deg.to_radians(), pitch_deg.to_radians(), 0.0);
                }
            }

            // Yaw
            let mut yaw_prop = Property {
                name: "Yaw".to_string(),
                value: PropertyValue::Float(yaw_deg),
                metadata: PropertyMetadata {
                    step: Some(5.0),
                    ..default()
                },
            };

            let editor = state.get_or_create_lighting_editor("dir_yaw", &yaw_prop.value);
            if editor.render(ui, &mut yaw_prop) {
                if let PropertyValue::Float(v) = yaw_prop.value {
                    yaw_deg = v;
                    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw_deg.to_radians(), pitch_deg.to_radians(), 0.0);
                }
            }
        });
    }

    ui.add_space(8.0);

    // Ambient Light controls
    ui.group(|ui| {
        ui.label("Ambient Light:");

        // Brightness
        let mut brightness_prop = Property {
            name: "Brightness".to_string(),
            value: PropertyValue::Float(ambient_light.brightness),
            metadata: PropertyMetadata {
                step: Some(50.0),
                min: Some(0.0),
                ..default()
            },
        };

        let editor = state.get_or_create_lighting_editor("ambient_brightness", &brightness_prop.value);
        if editor.render(ui, &mut brightness_prop) {
            if let PropertyValue::Float(v) = brightness_prop.value {
                ambient_light.brightness = v;
            }
        }

        // Color
        let mut color_prop = Property {
            name: "Color".to_string(),
            value: PropertyValue::Color(ambient_light.color),
            metadata: PropertyMetadata::default(),
        };

        let editor = state.get_or_create_lighting_editor("ambient_color", &color_prop.value);
        if editor.render(ui, &mut color_prop) {
            if let PropertyValue::Color(c) = color_prop.value {
                ambient_light.color = c;
            }
        }
    });
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::srgb(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0
    ))
}

// ============================================================================
// Plugin
// ============================================================================

/// Initialize the inspector registry with all inspectable components
pub fn init_inspector_registry(app: &mut App) {
    app.init_resource::<InspectorRegistry>();

    let mut registry = app.world_mut().resource_mut::<InspectorRegistry>();
    registry.register::<Transform>();
    registry.register::<RigidBodyType>();
    registry.register::<PointLight>();
    registry.register::<SpotLight>();
    // Add more components as needed
}
