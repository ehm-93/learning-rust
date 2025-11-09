use bevy::prelude::*;
use bevy::picking::events::{Pointer, Click};

use super::components::EditorEntity;
use super::placement::PlacementState;

/// Resource tracking the currently selected entity
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
    mut selected: ResMut<SelectedEntity>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    placement_state: Res<PlacementState>,
    editor_query: Query<(), With<EditorEntity>>,
) {
    // Don't select if in placement mode
    if placement_state.active {
        return;
    }

    let clicked_entity = trigger.target();

    // Only select EditorEntity objects
    if editor_query.get(clicked_entity).is_err() {
        return;
    }

    // Clear previous selection
    for entity in selected_query.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    // Select the clicked entity
    selected.entity = Some(clicked_entity);
    commands.entity(clicked_entity).insert(Selected);
    info!("Selected entity: {:?}", clicked_entity);
}

/// Handle deselection (ESC key or click on empty space)
pub fn handle_deselection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedEntity>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    placement_state: Res<PlacementState>,
) {
    // Don't deselect if in placement mode (ESC cancels placement instead)
    if placement_state.active {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) && selected.entity.is_some() {
        // Clear selection
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        selected.entity = None;
        info!("Deselected");
    }
}

/// Add visual outline to selected entities
pub fn highlight_selected(
    mut commands: Commands,
    selected_query: Query<Entity, (With<Selected>, Without<Outline>)>,
) {
    for entity in selected_query.iter() {
        commands.entity(entity).insert(Outline {
            color: Color::srgb(1.0, 0.8, 0.0), // Yellow outline
            offset: Val::Px(2.0),
            width: Val::Px(3.0),
        });
    }
}

/// Remove outline from deselected entities
pub fn remove_outline_from_deselected(
    mut commands: Commands,
    outline_query: Query<Entity, (With<Outline>, Without<Selected>)>,
) {
    for entity in outline_query.iter() {
        commands.entity(entity).remove::<Outline>();
    }
}
