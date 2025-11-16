//! Viewport plugin for 3D scene viewing and navigation

use bevy::prelude::*;

use super::camera::{setup_editor_camera, toggle_mouse_lock, camera_look, camera_movement, lock_cursor_on_start};
use super::grid::{GridConfig, setup_grid, toggle_snap};

/// Resource to track whether custom lighting is enabled
#[derive(Resource)]
pub struct LightingEnabled(pub bool);

impl Default for LightingEnabled {
    fn default() -> Self {
        Self(true)
    }
}

/// Stores saved custom lighting values when in simple mode
#[derive(Resource)]
pub struct SavedLightingState {
    pub dir_illuminance: f32,
    pub dir_color: Color,
    pub ambient_brightness: f32,
    pub ambient_color: Color,
}

impl Default for SavedLightingState {
    fn default() -> Self {
        Self {
            dir_illuminance: 10000.0,
            dir_color: Color::WHITE,
            ambient_brightness: 400.0,
            ambient_color: Color::WHITE,
        }
    }
}

/// Plugin for viewport functionality (camera, grid, raycasting)
pub struct ViewportPlugin;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GridConfig>()
            .init_resource::<LightingEnabled>()
            .init_resource::<SavedLightingState>()

            // Startup systems
            .add_systems(Startup, (
                setup_editor_camera,
                setup_grid,
                setup_lighting,
                lock_cursor_on_start,
            ))

            // Update systems
            .add_systems(Update, (
                toggle_mouse_lock,
                camera_look,
                camera_movement,
                toggle_snap,
                update_lighting_mode,
            ));
    }
}

/// Setup default lighting for the editor
fn setup_lighting(mut commands: Commands) {
    // Directional light (sun-like)
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light (fill light)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
        ..default()
    });
}

/// System to update lighting based on enabled state
fn update_lighting_mode(
    lighting_enabled: Res<LightingEnabled>,
    mut saved_state: ResMut<SavedLightingState>,
    mut directional_lights: Query<&mut DirectionalLight>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    if !lighting_enabled.is_changed() {
        return;
    }

    if lighting_enabled.0 {
        // Switching TO custom lighting mode - restore from saved state
        for mut dir_light in directional_lights.iter_mut() {
            dir_light.illuminance = saved_state.dir_illuminance;
            dir_light.color = saved_state.dir_color;
        }
        ambient_light.brightness = saved_state.ambient_brightness;
        ambient_light.color = saved_state.ambient_color;
    } else {
        // Switching to simple mode - save current custom values BEFORE changing
        if let Ok(dir_light) = directional_lights.single() {
            saved_state.dir_illuminance = dir_light.illuminance;
            saved_state.dir_color = dir_light.color;
        }
        saved_state.ambient_brightness = ambient_light.brightness;
        saved_state.ambient_color = ambient_light.color;

        // Now apply simple mode: bright white ambient, no directional
        for mut dir_light in directional_lights.iter_mut() {
            dir_light.illuminance = 0.0; // Disable directional light
        }
        ambient_light.brightness = 10000.0;
        ambient_light.color = Color::WHITE;
    }
}
