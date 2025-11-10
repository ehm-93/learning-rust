//! Save and load systems triggered by keyboard shortcuts

use bevy::prelude::*;
use std::path::PathBuf;

use crate::editor::persistence::scene::save_scene;
use crate::editor::core::types::EditorEntity;
use crate::editor::ui::menu_bar::{SaveAsEvent, OpenFileEvent};
use crate::editor::ui::confirmation_dialog::{ConfirmationDialog, PendingAction};

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
