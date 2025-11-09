use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use super::components::{EditorCamera, EditorEntity};
use super::resources::EditorMouseMotion;

/// Setup the editor camera when entering editor mode
pub fn setup_editor_camera(mut commands: Commands) {
    // Spawn editor camera at a good starting position
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        EditorCamera::default(),
    ));
}

/// Setup a simple test scene
pub fn setup_test_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        EditorEntity,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Some test cubes
    for i in -2..3 {
        for j in -2..3 {
            if i == 0 && j == 0 {
                continue; // Skip center
            }
            commands.spawn((
                EditorEntity,
                Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb(
                    0.7 + i as f32 * 0.1,
                    0.5,
                    0.7 + j as f32 * 0.1,
                ))),
                Transform::from_xyz(i as f32 * 3.0, 1.0, j as f32 * 3.0),
            ));
        }
    }

    // Directional light
    commands.spawn((
        EditorEntity,
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
    });
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

/// Lock cursor when editor starts
pub fn lock_cursor_on_start(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Confined;
        window.cursor_options.visible = false;
    }
}
