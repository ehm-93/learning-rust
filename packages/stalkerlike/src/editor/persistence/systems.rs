//! Save and load systems triggered by keyboard shortcuts

use bevy::prelude::*;
use std::path::PathBuf;
use std::env;

use crate::editor::persistence::scene::{save_scene, load_scene};
use crate::editor::core::types::EditorEntity;

/// Get the base directory for stalkerlike data
/// Checks STALKERLIKE_HOME environment variable, falls back to current working directory
fn get_stalkerlike_home() -> PathBuf {
    if let Ok(home) = env::var("STALKERLIKE_HOME") {
        PathBuf::from(home)
    } else {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

/// Get the default scene path relative to STALKERLIKE_HOME
fn get_default_scene_path() -> PathBuf {
    let home = get_stalkerlike_home();
    home.join("assets").join("levels").join("test_scene.yaml")
}

/// Resource tracking the current scene file
#[derive(Resource, Default)]
pub struct CurrentFile {
    pub path: Option<PathBuf>,
    pub dirty: bool,
}

impl CurrentFile {
    /// Get the current file path or the default path
    pub fn get_path(&self) -> PathBuf {
        self.path.clone().unwrap_or_else(get_default_scene_path)
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
            // Skip save if no file is open
            if !current_file.has_path() {
                warn!("No file open, cannot save. Use Save As... instead.");
                return;
            }

            // Use current path or default relative to STALKERLIKE_HOME
            let path = current_file.get_path();

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
pub fn load_scene_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut current_file: ResMut<CurrentFile>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    editor_entities: Query<Entity, With<EditorEntity>>,
) {
    // Check for Ctrl+O
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::KeyO) {
            // Use current path or default relative to STALKERLIKE_HOME
            let path = current_file.get_path();

            // Check if file exists
            if !path.exists() {
                warn!("Scene file does not exist: {}", path.display());
                return;
            }

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
                }
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

/// Startup system to log the STALKERLIKE_HOME directory
pub fn log_stalkerlike_home() {
    let home = get_stalkerlike_home();
    info!("STALKERLIKE_HOME: {}", home.display());

    let default_path = get_default_scene_path();
    info!("Default scene path: {}", default_path.display());
}
