use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::editor::persistence::{AutoSaveInterval, AutoSaveTimer, CurrentFile, NewFileEvent, OpenFileEvent, SaveEvent, SaveAsEvent};
use crate::editor::ui::confirmation_dialog::{ConfirmationDialog, PendingAction};
use crate::editor::ui::shortcuts::ShortcutsPanel;

/// Render the top menu bar
pub fn menu_bar_ui(
    mut contexts: EguiContexts,
    current_file: Res<CurrentFile>,
    mut autosave_timer: ResMut<AutoSaveTimer>,
    mut dialog: ResMut<ConfirmationDialog>,
    mut shortcuts_panel: ResMut<ShortcutsPanel>,
    mut new_file_events: EventWriter<NewFileEvent>,
    mut open_file_events: EventWriter<OpenFileEvent>,
    mut save_events: EventWriter<SaveEvent>,
    mut save_as_events: EventWriter<SaveAsEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Handle keyboard shortcuts
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if ctrl_held && keyboard.just_pressed(KeyCode::KeyN) {
        // Ctrl+N: New file
        if current_file.is_dirty() {
            dialog.request(PendingAction::NewFile);
        } else {
            new_file_events.write(NewFileEvent);
        }
    }

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New File (Ctrl+N)").clicked() {
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

                ui.separator();

                // Autosave submenu
                ui.menu_button(format!("Autosave ({})", autosave_timer.interval.label()), |ui| {
                    for interval in AutoSaveInterval::all() {
                        let is_selected = autosave_timer.interval == *interval;
                        let label = if is_selected {
                            format!("✓ {}", interval.label())
                        } else {
                            interval.label().to_string()
                        };

                        if ui.button(label).clicked() {
                            autosave_timer.set_interval(*interval);
                            info!("Autosave interval set to: {}", interval.label());
                            ui.close();
                        }
                    }
                });
            });

            ui.menu_button("Help", |ui| {
                if ui.button("Keyboard Shortcuts (F1)").clicked() {
                    shortcuts_panel.toggle();
                    ui.close();
                }
            });
        });
    });
}
