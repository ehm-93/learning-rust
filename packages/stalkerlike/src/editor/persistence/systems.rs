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

/// Resource tracking the current scene file path
#[derive(Resource, Default)]
pub struct SceneFile {
    pub path: Option<PathBuf>,
    pub dirty: bool,
}

/// System to handle save scene keyboard shortcut (Ctrl+S)
pub fn save_scene_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scene_file: ResMut<SceneFile>,
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
            // Use current path or default relative to STALKERLIKE_HOME
            let path = scene_file.path.clone().unwrap_or_else(get_default_scene_path);

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
                    scene_file.path = Some(path);
                    scene_file.dirty = false;
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
    mut scene_file: ResMut<SceneFile>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    editor_entities: Query<Entity, With<EditorEntity>>,
) {
    // Check for Ctrl+O
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::KeyO) {
            // Use current path or default relative to STALKERLIKE_HOME
            let path = scene_file.path.clone().unwrap_or_else(get_default_scene_path);

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
                    scene_file.path = Some(path);
                    scene_file.dirty = false;
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
    mut scene_file: ResMut<SceneFile>,
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
        scene_file.dirty = true;
    }
}

/// Startup system to log the STALKERLIKE_HOME directory
pub fn log_stalkerlike_home() {
    let home = get_stalkerlike_home();
    info!("STALKERLIKE_HOME: {}", home.display());

    let default_path = get_default_scene_path();
    info!("Default scene path: {}", default_path.display());
}
