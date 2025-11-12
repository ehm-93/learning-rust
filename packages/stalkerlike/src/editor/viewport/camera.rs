//! Editor camera system with fly controls
//!
//! This module provides a free-flying camera controller for navigating the 3D viewport.
//! The camera uses WASD+QE for movement and mouse look for rotation, similar to
//! game editors like Unity, Unreal, and Blender.
//!
//! # Controls
//!
//! - **WASD**: Move forward/left/back/right
//! - **QE**: Move down/up
//! - **Mouse**: Look around (when mouse is locked)
//! - **Left Alt**: Toggle mouse lock (permanent)
//! - **Middle Mouse Button**: Temporarily enable mouse look while held
//! - **Shift**: Hold for 4x speed boost
//!
//! # Mouse Lock Modes
//!
//! - **Unlocked** (default): Cursor visible, can interact with UI
//! - **Locked**: Cursor hidden/confined, camera responds to mouse movement
//! - **Temporary**: Middle mouse button provides temporary lock without toggling state

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::editor::input::mouse::EditorMouseMotion;

/// Editor camera controller with fly-around controls
/// Attach to the same entity as Camera3d - this just adds editor-specific behavior
#[derive(Component)]
pub struct EditorCamera {
    /// Mouse look sensitivity
    pub sensitivity: f32,
    /// Vertical rotation (pitch) in radians
    pub pitch: f32,
    /// Horizontal rotation (yaw) in radians
    pub yaw: f32,
    /// Current velocity for smooth movement
    pub velocity: Vec3,
    /// Base movement speed (units per second)
    pub base_speed: f32,
    /// Whether mouse is locked for camera control (true) or free for UI interaction (false)
    pub mouse_locked: bool,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            base_speed: 5.0,
            mouse_locked: true,
        }
    }
}

/// Setup the editor camera when entering editor mode
pub fn setup_editor_camera(mut commands: Commands) {
    // Spawn editor camera at an isometric view of the origin
    // Position: 45° horizontally, elevated for classic isometric angle
    let distance = 15.0;
    let yaw = -std::f32::consts::FRAC_PI_4; // -45 degrees (southwest direction)
    let pitch = -35.264_f32.to_radians(); // Looking down at isometric angle

    let x = distance * pitch.cos() * yaw.sin();
    let y = distance * (-pitch).sin();
    let z = distance * pitch.cos() * yaw.cos();

    let transform = Transform::from_xyz(x, y, z)
        .looking_at(Vec3::ZERO, Vec3::Y);

    let mut camera = EditorCamera::default();
    camera.yaw = yaw;
    camera.pitch = pitch;
    camera.mouse_locked = false; // Start with mouse unlocked for UI interaction

    commands.spawn((
        Camera3d::default(),
        transform,
        camera,
    ));
}

/// Toggle mouse lock mode with Left Alt or temporarily lock with Middle Mouse Button
pub fn toggle_mouse_lock(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<&mut EditorCamera>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    // Toggle lock state with Alt key
    if keyboard.just_pressed(KeyCode::AltLeft) {
        for mut camera in camera_query.iter_mut() {
            camera.mouse_locked = !camera.mouse_locked;
        }
    }

    // Update cursor state based on permanent lock OR middle mouse button
    for camera in camera_query.iter() {
        let middle_mouse_held = mouse.pressed(MouseButton::Middle);
        let should_lock = camera.mouse_locked || middle_mouse_held;

        if let Ok(mut window) = windows.single_mut() {
            if should_lock {
                window.cursor_options.grab_mode = CursorGrabMode::Confined;
                window.cursor_options.visible = false;
            } else {
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
            }
        }
    }
}

/// Handle mouse look when mouse is locked or middle mouse button is held
pub fn camera_look(
    mut motion_evr: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_motion: ResMut<EditorMouseMotion>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<(&mut EditorCamera, &mut Transform)>,
) {
    // Accumulate mouse motion
    for ev in motion_evr.read() {
        mouse_motion.delta += ev.delta;
    }

    for (mut camera, mut transform) in camera_query.iter_mut() {
        // Apply mouse look if mouse is locked OR middle mouse button is held
        let middle_mouse_held = mouse.pressed(MouseButton::Middle);
        if !camera.mouse_locked && !middle_mouse_held {
            mouse_motion.delta = Vec2::ZERO;
            continue;
        }

        // Update rotation based on mouse movement
        camera.yaw -= mouse_motion.delta.x * camera.sensitivity;
        camera.pitch -= mouse_motion.delta.y * camera.sensitivity;

        // Clamp pitch to prevent camera flipping
        camera.pitch = camera.pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);

        // Apply rotation to transform
        transform.rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);
    }

    mouse_motion.delta = Vec2::ZERO;
}

/// Handle WASD + QE movement with velocity-based smooth movement
pub fn camera_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<(&mut EditorCamera, &mut Transform)>,
) {
    for (mut camera, mut transform) in camera_query.iter_mut() {
        // Get movement input
        let mut input_direction = Vec3::ZERO;

        // Forward/backward (W/S)
        if keyboard.pressed(KeyCode::KeyW) {
            input_direction += Vec3::NEG_Z;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            input_direction += Vec3::Z;
        }

        // Left/right (A/D)
        if keyboard.pressed(KeyCode::KeyA) {
            input_direction += Vec3::NEG_X;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            input_direction += Vec3::X;
        }

        // Up/down (E/Q)
        if keyboard.pressed(KeyCode::KeyE) {
            input_direction += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::KeyQ) {
            input_direction += Vec3::NEG_Y;
        }

        // Calculate speed multipliers
        let mut speed_multiplier = 1.0;
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            speed_multiplier = 4.0;
        } else if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            speed_multiplier = 0.25;
        }

        // Normalize input direction and apply speed
        let target_velocity = if input_direction.length() > 0.0 {
            input_direction.normalize() * camera.base_speed * speed_multiplier
        } else {
            Vec3::ZERO
        };

        // Smooth velocity interpolation
        let smoothing = 10.0; // Higher = more responsive
        camera.velocity = camera.velocity.lerp(target_velocity, smoothing * time.delta_secs());

        // Transform velocity to world space based on camera rotation
        let forward = transform.forward();
        let right = transform.right();
        let up = Vec3::Y; // Always use world up for vertical movement

        let world_velocity =
            forward * -camera.velocity.z +
            right * camera.velocity.x +
            up * camera.velocity.y;

        // Apply velocity to position
        transform.translation += world_velocity * time.delta_secs();
    }
}

/// Unlock cursor when editor starts (for UI interaction)
pub fn lock_cursor_on_start(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}
