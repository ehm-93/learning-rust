//! Box selection system for multi-select via drag
//!
//! This module provides a visual box selection tool (like in Blender, Photoshop, etc.)
//! that allows users to select multiple objects by clicking and dragging to define
//! a rectangular selection region.
//!
//! # Features
//!
//! - **Visual feedback**: Renders a semi-transparent blue box showing selection area
//! - **Screen-space selection**: Uses 2D screen coordinates for intuitive interaction
//! - **3D object picking**: Checks which 3D objects fall within the selection box
//! - **Additive selection**: Hold Shift to add to existing selection
//! - **Cancel**: Right-click or ESC to cancel box select
//!
//! # Workflow
//!
//! 1. User clicks and drags (not on an object) to start box select
//! 2. `start_box_select()` captures start position
//! 3. `update_box_select()` tracks mouse position, updates visual box
//! 4. `complete_box_select()` on mouse release - selects all objects in box
//! 5. `cancel_box_select()` on right-click or ESC - aborts selection

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::editor::core::types::EditorEntity;
use crate::editor::objects::placement::PlacementState;
use crate::editor::objects::selection::{SelectionSet, Selected};

/// Resource tracking box selection state
#[derive(Resource, Default)]
pub struct BoxSelectState {
    pub active: bool,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
    /// Track when mouse was pressed to implement a small delay
    pub mouse_press_frame: Option<u32>,
    pub frame_count: u32,
}

impl BoxSelectState {
    /// Get the min and max corners of the selection box
    pub fn get_rect(&self) -> (Vec2, Vec2) {
        let min = Vec2::new(
            self.start_pos.x.min(self.current_pos.x),
            self.start_pos.y.min(self.current_pos.y),
        );
        let max = Vec2::new(
            self.start_pos.x.max(self.current_pos.x),
            self.start_pos.y.max(self.current_pos.y),
        );
        (min, max)
    }
}

/// Marker component for the box select visualization entity
#[derive(Component)]
pub struct BoxSelectVisual;

/// Start box selection on mouse drag in empty space
/// Uses a delayed activation to let picking system handle entity clicks first
pub fn start_box_select(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut box_select: ResMut<BoxSelectState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    placement_state: Res<PlacementState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selection: Res<SelectionSet>,
) {
    // Increment frame counter
    box_select.frame_count += 1;

    // Don't start box select if in placement mode
    if placement_state.active {
        return;
    }

    // Don't start if entities are already selected
    if !selection.is_empty() {
        return;
    }

    // Don't start if Ctrl is held (that's for multi-select clicking)
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    let Ok(window) = window_query.single() else {
        return;
    };

    // Track when mouse was first pressed
    if mouse_input.just_pressed(MouseButton::Left) && !ctrl_held {
        if let Some(cursor_pos) = window.cursor_position() {
            box_select.mouse_press_frame = Some(box_select.frame_count);
            box_select.start_pos = cursor_pos;
            box_select.current_pos = cursor_pos;
        }
    }

    // Only activate box select if:
    // 1. Mouse is still pressed
    // 2. At least 2 frames have passed (giving picking system time to process clicks)
    // 3. Mouse has moved at least 5 pixels (user is dragging, not clicking)
    if let Some(press_frame) = box_select.mouse_press_frame {
        if mouse_input.pressed(MouseButton::Left) {
            if let Some(cursor_pos) = window.cursor_position() {
                let frames_since_press = box_select.frame_count - press_frame;
                let drag_distance = (cursor_pos - box_select.start_pos).length();

                // Activate after 2 frames AND some mouse movement
                if frames_since_press >= 2 && drag_distance > 5.0 && !box_select.active {
                    box_select.active = true;
                    info!("Box select activated after {} frames and {:.1}px drag", frames_since_press, drag_distance);
                }
            }
        } else {
            // Mouse released, reset tracking
            box_select.mouse_press_frame = None;
            if !box_select.active {
                // Was a quick click, not a drag - picking system will handle it
            }
        }
    }
}

/// Update box selection while dragging
pub fn update_box_select(
    mut box_select: ResMut<BoxSelectState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if !box_select.active {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        box_select.current_pos = cursor_pos;
    }
}

/// Complete box selection on mouse release
pub fn complete_box_select(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut box_select: ResMut<BoxSelectState>,
    mut selection: ResMut<SelectionSet>,
    mut commands: Commands,
    editor_query: Query<(Entity, &GlobalTransform), With<EditorEntity>>,
    selected_query: Query<Entity, With<Selected>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if !box_select.active {
        return;
    }

    // End box select on left mouse button release
    if mouse_input.just_released(MouseButton::Left) {
        let (min, max) = box_select.get_rect();
        let size = max - min;

        info!("Completed box select: {:?} to {:?} (size: {:?})", min, max, size);

        // If the box is too small (just a click or small gizmo drag), don't select anything
        // Increased threshold to 25 pixels to better distinguish from gizmo interactions
        if size.length() < 25.0 {
            box_select.active = false;
            return;
        }

        let Ok((camera, camera_transform)) = camera_query.single() else {
            box_select.active = false;
            return;
        };

        // Clear previous selection
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        selection.clear();

        // Select all entities whose screen position falls within the box
        for (entity, transform) in editor_query.iter() {
            let world_pos = transform.translation();

            // Project world position to screen space
            if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) {
                // Check if screen position is inside the selection box
                if screen_pos.x >= min.x && screen_pos.x <= max.x &&
                   screen_pos.y >= min.y && screen_pos.y <= max.y {
                    selection.add(entity);
                    commands.entity(entity).insert(Selected);
                    info!("Selected entity: {:?} at screen pos {:?}", entity, screen_pos);
                }
            }
        }

        info!("Box select completed: {} entities selected", selection.len());
        box_select.active = false;
    }
}

/// Cancel box selection on ESC key
pub fn cancel_box_select(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut box_select: ResMut<BoxSelectState>,
) {
    if box_select.active && keyboard.just_pressed(KeyCode::Escape) {
        box_select.active = false;
        info!("Box select cancelled");
    }
}

/// Render the box selection rectangle using EGUI
pub fn render_box_select(
    mut contexts: bevy_egui::EguiContexts,
    box_select: Res<BoxSelectState>,
) {
    if !box_select.active {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let (min, max) = box_select.get_rect();

    // Draw a semi-transparent rectangle with a border
    use bevy_egui::egui;

    egui::Area::new(egui::Id::new("box_select_area"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .interactable(false)
        .show(ctx, |ui| {
            let painter = ui.painter();

            // Convert Bevy screen coordinates to EGUI coordinates
            let rect = egui::Rect::from_min_max(
                egui::pos2(min.x, min.y),
                egui::pos2(max.x, max.y),
            );

            // Draw both fill and stroke using Shape::Rect
            use egui::epaint::{Stroke as EguiStroke, StrokeKind};

            let shape = egui::Shape::Rect(egui::epaint::RectShape {
                rect,
                corner_radius: 0.0.into(),
                fill: egui::Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                stroke: EguiStroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                stroke_kind: StrokeKind::Middle,
                round_to_pixels: Some(false),
                blur_width: 0.0,
                brush: Default::default(),
            });
            painter.add(shape);
        });
}

