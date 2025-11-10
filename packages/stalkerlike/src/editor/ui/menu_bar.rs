use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::persistence::CurrentFile;
use crate::editor::persistence::scene::{save_scene, load_scene};
use crate::editor::core::types::EditorEntity;
use crate::editor::ui::confirmation_dialog::{ConfirmationDialog, ErrorDialog, PendingAction};

/// Event to trigger a new file
#[derive(Event)]
pub struct NewFileEvent;

/// Event to trigger open file dialog
#[derive(Event)]
pub struct OpenFileEvent;

/// Event to trigger save
#[derive(Event)]
pub struct SaveEvent;

/// Event to trigger save as dialog
#[derive(Event)]
pub struct SaveAsEvent;

/// Render the top menu bar
pub fn menu_bar_ui(
    mut contexts: EguiContexts,
    current_file: Res<CurrentFile>,
    mut dialog: ResMut<ConfirmationDialog>,
    mut new_file_events: EventWriter<NewFileEvent>,
    mut open_file_events: EventWriter<OpenFileEvent>,
    mut save_events: EventWriter<SaveEvent>,
    mut save_as_events: EventWriter<SaveAsEvent>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New File").clicked() {
                    if current_file.is_dirty() {
                        dialog.request(PendingAction::NewFile);
                    } else {
                        new_file_events.write(NewFileEvent);
                    }
                    ui.close();
                }

                if ui.button("Open File...").clicked() {
                    if current_file.is_dirty() {
                        dialog.request(PendingAction::OpenFile);
                    } else {
                        open_file_events.write(OpenFileEvent);
                    }
                    ui.close();
                }

                ui.separator();

                // Disable Save if no file is open
                if ui.add_enabled(current_file.has_path(), egui::Button::new("Save")).clicked() {
                    save_events.write(SaveEvent);
                    ui.close();
                }

                if ui.button("Save As...").clicked() {
                    save_as_events.write(SaveAsEvent);
                    ui.close();
                }
            });
        });
    });
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
