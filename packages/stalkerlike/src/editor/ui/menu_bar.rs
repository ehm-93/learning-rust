use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::persistence::{CurrentFile, NewFileEvent, OpenFileEvent, SaveEvent, SaveAsEvent};
use crate::editor::ui::confirmation_dialog::{ConfirmationDialog, PendingAction};

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
