//! Save and load systems triggered by keyboard shortcuts or UI events

use bevy::prelude::*;
use std::path::PathBuf;

use crate::editor::persistence::scene::{save_scene, load_scene};
use crate::editor::persistence::events::{NewFileEvent, OpenFileEvent, SaveEvent, SaveAsEvent};
use crate::editor::core::types::EditorEntity;
use crate::editor::ui::confirmation_dialog::{ConfirmationDialog, ErrorDialog, PendingAction};

/// Resource tracking the current scene file
#[derive(Resource, Default)]
pub struct CurrentFile {
    pub path: Option<PathBuf>,
    pub dirty: bool,
}

impl CurrentFile {
    /// Get the current file path or the default path
    pub fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    /// Set the current file path
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    /// Mark the file as clean (saved)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Mark the file as dirty (unsaved changes)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if there are unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if a file path is set
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }

    /// Get the filename for display
    pub fn get_filename(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
            .unwrap_or("untitled")
            .to_string()
    }
}

/// System to handle save scene keyboard shortcut (Ctrl+S)
pub fn save_scene_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_file: ResMut<CurrentFile>,
    mut save_as_events: EventWriter<SaveAsEvent>,
    editor_entities: Query<(
        Entity,
        &Transform,
        Option<&Name>,
        Option<&Mesh3d>,
        Option<&MeshMaterial3d<StandardMaterial>>,
    ), With<EditorEntity>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    // Check for Ctrl+S
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::KeyS) {
            // If no file is open, trigger Save As dialog instead
            if !current_file.has_path() {
                info!("No file open, opening Save As dialog...");
                save_as_events.write(SaveAsEvent);
                return;
            }

            // Get the current file path (we know it exists from has_path check)
            let Some(path) = current_file.get_path() else {
                return;
            };

            // Ensure the directory exists
            if let Some(parent) = path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    error!("Failed to create directory {}: {}", parent.display(), e);
                    return;
                }
            }

            match save_scene(path.clone(), editor_entities, meshes, materials) {
                Ok(()) => {
                    info!("Scene saved to {}", path.display());
                    current_file.set_path(path);
                    current_file.mark_clean();
                }
                Err(e) => {
                    error!("Failed to save scene: {}", e);
                }
            }
        }
    }
}

/// System to handle load scene keyboard shortcut (Ctrl+O)
/// Opens the file picker dialog to select a scene file
pub fn load_scene_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_file: Res<CurrentFile>,
    mut dialog: ResMut<ConfirmationDialog>,
    mut open_file_events: EventWriter<OpenFileEvent>,
) {
    // Check for Ctrl+O
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::KeyO) {
            // Check for unsaved changes
            if current_file.is_dirty() {
                dialog.request(PendingAction::OpenFile);
            } else {
                // Trigger the Open File dialog
                info!("Opening file picker dialog...");
                open_file_events.write(OpenFileEvent);
            }
        }
    }
}

/// System to mark scene as dirty when entities are modified
pub fn mark_scene_dirty(
    mut current_file: ResMut<CurrentFile>,
    changed_entities: Query<
        Entity,
        (
            With<EditorEntity>,
            Or<(
                Changed<Transform>,
                Changed<Mesh3d>,
                Changed<MeshMaterial3d<StandardMaterial>>,
            )>,
        ),
    >,
) {
    if !changed_entities.is_empty() {
        current_file.mark_dirty();
    }
}

/// System to handle new file event
pub fn handle_new_file(
    mut events: EventReader<NewFileEvent>,
    mut current_file: ResMut<CurrentFile>,
    mut commands: Commands,
    editor_entities: Query<Entity, With<EditorEntity>>,
) {
    for _ in events.read() {
        // Clear the scene
        for entity in editor_entities.iter() {
            commands.entity(entity).despawn();
        }

        // Reset current file
        current_file.path = None;
        current_file.mark_clean();

        info!("New file created");
    }
}

/// System to handle save event
pub fn handle_save(
    mut events: EventReader<SaveEvent>,
    mut current_file: ResMut<CurrentFile>,
    mut error_dialog: ResMut<ErrorDialog>,
    editor_entities: Query<(
        Entity,
        &Transform,
        Option<&Name>,
        Option<&Mesh3d>,
        Option<&MeshMaterial3d<StandardMaterial>>,
    ), With<EditorEntity>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    for _ in events.read() {
        // Skip save if no file is open - should not happen since button is disabled
        if !current_file.has_path() {
            warn!("No file open, cannot save.");
            continue;
        }

        let Some(path) = current_file.get_path() else {
            continue;
        };

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!("Failed to create directory {}: {}", parent.display(), e);
                error_dialog.show_error(
                    "Save Failed",
                    format!("Failed to create directory {}:\n{}", parent.display(), e)
                );
                return;
            }
        }

        match save_scene(path.clone(), editor_entities, Res::clone(&meshes), Res::clone(&materials)) {
            Ok(()) => {
                info!("Scene saved to {}", path.display());
                current_file.set_path(path);
                current_file.mark_clean();
            }
            Err(e) => {
                error!("Failed to save scene: {}", e);
                error_dialog.show_error(
                    "Save Failed",
                    format!("Failed to save scene:\n{}", e)
                );
            }
        }
    }
}

/// System to handle open file event
/// Opens a file picker dialog to select a scene file
pub fn handle_open_file(
    mut events: EventReader<OpenFileEvent>,
    mut commands: Commands,
    mut current_file: ResMut<CurrentFile>,
    mut error_dialog: ResMut<ErrorDialog>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    editor_entities: Query<Entity, With<EditorEntity>>,
) {
    for _ in events.read() {
        // Open file picker dialog
        let file_dialog = rfd::FileDialog::new()
            .add_filter("Scene Files", &["yaml", "yml"])
            .set_title("Open Scene");

        if let Some(path) = file_dialog.pick_file() {
            // Clear existing scene entities
            for entity in editor_entities.iter() {
                commands.entity(entity).despawn();
            }

            match load_scene(path.clone(), &mut commands, &mut meshes, &mut materials) {
                Ok(()) => {
                    info!("Scene loaded from {}", path.display());
                    current_file.set_path(path);
                    current_file.mark_clean();
                }
                Err(e) => {
                    error!("Failed to load scene: {}", e);
                    error_dialog.show_error(
                        "Load Failed",
                        format!("Failed to load scene from {}:\n{}", path.display(), e)
                    );
                }
            }
        }
    }
}

/// System to handle save as event
/// Opens a file picker dialog to choose where to save the scene
pub fn handle_save_as(
    mut events: EventReader<SaveAsEvent>,
    mut current_file: ResMut<CurrentFile>,
    mut error_dialog: ResMut<ErrorDialog>,
    editor_entities: Query<(
        Entity,
        &Transform,
        Option<&Name>,
        Option<&Mesh3d>,
        Option<&MeshMaterial3d<StandardMaterial>>,
    ), With<EditorEntity>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    for _ in events.read() {
        // Open save file picker dialog
        let file_dialog = rfd::FileDialog::new()
            .add_filter("Scene Files", &["yaml", "yml"])
            .set_title("Save Scene As");

        if let Some(path) = file_dialog.save_file() {
            // Ensure the directory exists
            if let Some(parent) = path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    error!("Failed to create directory {}: {}", parent.display(), e);
                    error_dialog.show_error(
                        "Save Failed",
                        format!("Failed to create directory {}:\n{}", parent.display(), e)
                    );
                    return;
                }
            }

            match save_scene(path.clone(), editor_entities, Res::clone(&meshes), Res::clone(&materials)) {
                Ok(()) => {
                    info!("Scene saved as {}", path.display());
                    current_file.set_path(path);
                    current_file.mark_clean();
                }
                Err(e) => {
                    error!("Failed to save scene: {}", e);
                    error_dialog.show_error(
                        "Save Failed",
                        format!("Failed to save scene:\n{}", e)
                    );
                }
            }
        }
    }
}
