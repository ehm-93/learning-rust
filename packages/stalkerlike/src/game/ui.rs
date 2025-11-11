use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::resources::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // UI systems - run only in specific states
            .add_systems(EguiPrimaryContextPass,
                main_menu_ui.run_if(in_state(GameState::MainMenu)))
            .add_systems(EguiPrimaryContextPass,
                pause_menu_ui.run_if(in_state(GameState::Paused)))
            .add_systems(EguiPrimaryContextPass,
                loading_screen_ui.run_if(in_state(GameState::Loading)))
            .add_systems(EguiPrimaryContextPass,
                ingame_ui.run_if(in_state(GameState::InGame)))

            // Input handlers
            .add_systems(Update,
                handle_pause_input.run_if(in_state(GameState::InGame)))
            .add_systems(Update,
                handle_resume_input.run_if(in_state(GameState::Paused)))

            // State transition handlers
            .add_systems(OnEnter(GameState::NewGame), transition_to_ingame);
    }
}

fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut load_events: EventWriter<LoadGameEvent>,
) -> Result {
    egui::CentralPanel::default()
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Stalker-Like Game");
                ui.add_space(50.0);

                if ui.button("New Game").clicked() {
                    next_state.set(GameState::NewGame);
                }

                ui.add_space(10.0);

                if ui.button("Load Game").clicked() {
                    // Emit load event for slot 1 (default)
                    load_events.write(LoadGameEvent { slot: 1 });
                }

                ui.add_space(10.0);

                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });
        });
    Ok(())
}

fn pause_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
) -> Result {
    egui::Window::new("Paused")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                if ui.button("Resume").clicked() {
                    next_state.set(GameState::InGame);
                }

                ui.add_space(10.0);

                if ui.button("Save Game").clicked() {
                    // Emit save event for slot 1 (default)
                    save_events.write(SaveGameEvent { slot: 1 });
                    info!("Save game requested");
                }

                ui.add_space(10.0);

                if ui.button("Load Game").clicked() {
                    // Emit load event for slot 1 (default)
                    load_events.write(LoadGameEvent { slot: 1 });
                }

                ui.add_space(10.0);

                if ui.button("Main Menu").clicked() {
                    next_state.set(GameState::MainMenu);
                }

                ui.add_space(10.0);

                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }

                ui.add_space(10.0);
            });
        });
    Ok(())
}

fn ingame_ui(
    mut contexts: EguiContexts,
) -> Result {
    // Empty UI during gameplay - just consume the egui pass so bevy_egui doesn't error
    // We could add HUD elements here later
    let _ctx = contexts.ctx_mut()?;
    Ok(())
}

fn loading_screen_ui(
    mut contexts: EguiContexts,
    progress: Option<Res<LoadProgress>>,
) -> Result {
    egui::CentralPanel::default()
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(200.0);
                ui.heading("Loading...");
                ui.add_space(50.0);

                if let Some(progress) = progress {
                    // Progress bar
                    let progress_value = progress.progress();
                    ui.add(egui::ProgressBar::new(progress_value)
                        .text(format!("{:.0}%", progress_value * 100.0))
                        .desired_width(400.0)
                    );

                    ui.add_space(20.0);

                    // Show current system being loaded
                    if let Some(current) = progress.current_system() {
                        ui.label(format!("Loading: {}", current));
                    } else if progress.is_complete() {
                        ui.label("Complete!");
                    }

                    ui.add_space(20.0);

                    // Show all systems
                    ui.label(format!(
                        "Loaded {}/{} systems",
                        progress.completed.len(),
                        progress.registered.len()
                    ));
                } else {
                    ui.label("Initializing...");
                }
            });
        });
    Ok(())
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

fn handle_resume_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::InGame);
    }
}

fn transition_to_ingame(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}
