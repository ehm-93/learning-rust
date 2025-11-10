//! Confirmation dialog for unsaved changes

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::persistence::CurrentFile;
use crate::editor::ui::menu_bar::{NewFileEvent, OpenFileEvent, SaveEvent};

/// Pending action that requires confirmation
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    NewFile,
    OpenFile,
}

/// Resource to track if we need to show a confirmation dialog
#[derive(Resource, Default)]
pub struct ConfirmationDialog {
    pub show: bool,
    pub pending_action: Option<PendingAction>,
}

impl ConfirmationDialog {
    pub fn request(&mut self, action: PendingAction) {
        self.show = true;
        self.pending_action = Some(action);
    }

    pub fn close(&mut self) {
        self.show = false;
        self.pending_action = None;
    }
}

/// System to render the confirmation dialog
pub fn confirmation_dialog_ui(
    mut contexts: EguiContexts,
    mut dialog: ResMut<ConfirmationDialog>,
    current_file: Res<CurrentFile>,
    mut new_file_events: EventWriter<NewFileEvent>,
    mut open_file_events: EventWriter<OpenFileEvent>,
    mut save_events: EventWriter<SaveEvent>,
) {
    if !dialog.show {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut should_close = false;
    let mut should_save = false;
    let mut should_discard = false;

    egui::Window::new("Unsaved Changes")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "File '{}' has unsaved changes.",
                current_file.get_filename()
            ));
            ui.add_space(10.0);
            ui.label("Do you want to save your changes?");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    should_save = true;
                    should_close = true;
                }

                if ui.button("Don't Save").clicked() {
                    should_discard = true;
                    should_close = true;
                }

                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        if should_save {
            // Trigger save - if file has path, save it, otherwise use Save As
            save_events.write(SaveEvent);
        }
        
        if should_save || should_discard {
            // Proceed with the pending action (either after saving or discarding)
            if let Some(action) = dialog.pending_action {
                match action {
                    PendingAction::NewFile => {
                        new_file_events.write(NewFileEvent);
                    }
                    PendingAction::OpenFile => {
                        open_file_events.write(OpenFileEvent);
                    }
                }
            }
        }
        // If cancelled, just close without doing anything
        
        dialog.close();
    }
}
