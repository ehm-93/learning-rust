use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::resources::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(EguiPrimaryContextPass, main_menu_ui.run_if(in_state(GameState::MainMenu)))
            .add_systems(EguiPrimaryContextPass, pause_menu_ui.run_if(in_state(GameState::Paused)))
            .add_systems(Update, handle_pause_input.run_if(in_state(GameState::InGame)));
    }
}

fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    egui::CentralPanel::default()
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Stalker-Like Game");
                ui.add_space(50.0);

                if ui.button("New Game").clicked() {
                    next_state.set(GameState::InGame);
                }

                ui.add_space(10.0);

                if ui.button("Load Game").clicked() {
                    // TODO: Implement load game
                    next_state.set(GameState::InGame);
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
) -> Result {
    egui::CentralPanel::default().show(contexts.ctx_mut()?, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("Paused");
            ui.add_space(50.0);

            if ui.button("Resume").clicked() {
                next_state.set(GameState::InGame);
            }

            ui.add_space(10.0);

            if ui.button("Save Game").clicked() {
                // TODO: Implement save game
            }

            ui.add_space(10.0);

            if ui.button("Load Game").clicked() {
                // TODO: Implement load game
            }

            ui.add_space(10.0);

            if ui.button("Main Menu").clicked() {
                next_state.set(GameState::MainMenu);
            }

            ui.add_space(10.0);

            if ui.button("Exit").clicked() {
                std::process::exit(0);
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
