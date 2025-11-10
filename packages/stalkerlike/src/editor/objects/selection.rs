use bevy::prelude::*;
use bevy::picking::events::{Pointer, Click};
use std::collections::HashSet;

use crate::editor::core::types::EditorEntity;
use crate::editor::objects::placement::PlacementState;
use crate::editor::objects::outline::Outlined;
use crate::editor::objects::gizmo::GizmoHandle;

/// Resource tracking the set of currently selected entities
#[derive(Resource, Default)]
pub struct SelectionSet {
    pub entities: HashSet<Entity>,
}

impl SelectionSet {
    /// Check if an entity is selected
    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    /// Add an entity to the selection
    pub fn add(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    /// Remove an entity from the selection
    pub fn remove(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }

    /// Toggle an entity's selection state
    pub fn toggle(&mut self, entity: Entity) {
        if self.entities.contains(&entity) {
            self.entities.remove(&entity);
        } else {
            self.entities.insert(entity);
        }
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Get the first selected entity (for backward compatibility)
    pub fn first(&self) -> Option<Entity> {
        self.entities.iter().next().copied()
    }

    /// Check if the selection is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get the number of selected entities
    pub fn len(&self) -> usize {
        self.entities.len()
    }
}

// Legacy alias for backward compatibility - will be removed after migration
#[derive(Resource, Default)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
}

/// Marker component for selected entities
#[derive(Component)]
pub struct Selected;

/// Handle entity selection via picking Click events
pub fn handle_selection(
    trigger: Trigger<Pointer<Click>>,
    mut selection: ResMut<SelectionSet>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    placement_state: Res<PlacementState>,
    editor_query: Query<(), With<EditorEntity>>,
    gizmo_query: Query<(), With<GizmoHandle>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Don't select if in placement mode
    if placement_state.active {
        return;
    }

    let clicked_entity = trigger.target();

    // Don't select if clicking a gizmo handle
    if gizmo_query.get(clicked_entity).is_ok() {
        return;
    }

    // Only select EditorEntity objects
    if editor_query.get(clicked_entity).is_err() {
        return;
    }

    // Check if Ctrl is held for multi-select
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if ctrl_held {
        // Multi-select mode: toggle the clicked entity
        if selection.contains(clicked_entity) {
            // Remove from selection
            selection.remove(clicked_entity);
            commands.entity(clicked_entity).remove::<Selected>();
            info!("Removed from selection: {:?}", clicked_entity);
        } else {
            // Add to selection
            selection.add(clicked_entity);
            commands.entity(clicked_entity).insert(Selected);
            info!("Added to selection: {:?}", clicked_entity);
        }
    } else {
        // Single-select mode: clear previous selection and select only this entity
        // Clear previous selection
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        selection.clear();

        // Select the clicked entity
        selection.add(clicked_entity);
        commands.entity(clicked_entity).insert(Selected);
        info!("Selected entity: {:?}", clicked_entity);
    }
}

/// Handle deselection (ESC key or click on empty space)
pub fn handle_deselection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<SelectionSet>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    placement_state: Res<PlacementState>,
) {
    // Don't deselect if in placement mode (ESC cancels placement instead)
    if placement_state.active {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) && !selection.is_empty() {
        // Clear selection
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        selection.clear();
        info!("Deselected all");
    }
}

/// Add visual outline to selected entities
pub fn highlight_selected(
    mut commands: Commands,
    selected_query: Query<Entity, (With<Selected>, Without<Outlined>)>,
) {
    for entity in selected_query.iter() {
        commands.entity(entity).insert(Outlined::default());
    }
}

/// Remove outline from deselected entities
pub fn remove_outline_from_deselected(
    mut commands: Commands,
    outline_query: Query<Entity, (With<Outlined>, Without<Selected>)>,
) {
    for entity in outline_query.iter() {
        commands.entity(entity).remove::<Outlined>();
    }
}

/// Handle entity deletion (Delete key)
pub fn delete_selected(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<SelectionSet>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    placement_state: Res<PlacementState>,
) {
    // Don't delete if in placement mode
    if placement_state.active {
        return;
    }

    if keyboard.just_pressed(KeyCode::Delete) && !selection.is_empty() {
        // Delete all selected entities
        for entity in selected_query.iter() {
            info!("Deleting entity: {:?}", entity);
            commands.entity(entity).despawn();
        }

        // Clear selection
        selection.clear();
    }
}
