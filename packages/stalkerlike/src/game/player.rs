use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;
use bevy_egui::PrimaryEguiContext;

use super::components::*;
use super::resources::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MouseMotion>()

            // InGame state transitions
            .add_systems(OnEnter(GameState::InGame), cursor_grab)
            .add_systems(OnExit(GameState::InGame), cursor_release)

            // Player systems - only run during gameplay
            .add_systems(
                Update,
                (
                    player_movement,
                    camera_look,
                    toggle_flashlight,
                    update_flashlight_transform,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

pub fn setup_player(mut commands: Commands, spawn_position: Vec3) {
    // Player physics body at capsule center (1.8m tall human)
    commands.spawn((
        Player,
        Saveable,
        GameEntity,
        Health::default(),
        Transform::from_translation(spawn_position),  // Use provided spawn position
        PlayerCamera {
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        },
        RigidBody::Dynamic,
        Collider::capsule_y(0.65, 0.25),  // 1.8m tall (0.65*2 + 0.25*2), 0.5m diameter
        Velocity::default(),
        LockedAxes::ROTATION_LOCKED, // Prevent player from tipping over
    ))
    .with_children(|parent| {
        // Camera at eye height (offset up from capsule center)
        parent.spawn((
            Camera3d::default(),
            PrimaryEguiContext,
            Transform::from_xyz(0.0, 0.75, 0.0),  // Eye height above capsule center (~1.65m total)
        ))
        .with_children(|camera_parent| {
            // Flashlight as child of camera - offset to right and down like holding a flashlight
            camera_parent.spawn((
                Flashlight::default(),
                SpotLight {
                    intensity: 0.0,
                    range: 20.0,
                    radius: 0.50,  // Larger radius = softer shadows
                    shadows_enabled: true,
                    outer_angle: 0.3,
                    inner_angle: 0.2,
                    ..default()
                },
                Transform::from_xyz(0.3, -0.2, 0.0),  // Right and slightly down from camera
            ));
        });
    });
}

fn cursor_grab(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = q_windows.single_mut() {
        // for a game that doesn't use the cursor (like a shooter):
        window.cursor_options.grab_mode = CursorGrabMode::Confined;
        window.cursor_options.visible = false;
    }
}

fn cursor_release(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = q_windows.single_mut() {
        // Release the cursor when exiting the game
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &PlayerCamera), With<Player>>,
) {
    for (mut velocity, camera) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Get forward and right vectors from camera yaw
        let forward = Vec3::new(-camera.yaw.sin(), 0.0, -camera.yaw.cos());
        let right = Vec3::new(camera.yaw.cos(), 0.0, -camera.yaw.sin());

        if keyboard.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += right;
        }

        let speed = 5.0;
        let horizontal_velocity = if direction.length() > 0.0 {
            direction.normalize() * speed
        } else {
            Vec3::ZERO
        };

        // Only modify horizontal velocity, preserve vertical (gravity) velocity
        velocity.linvel.x = horizontal_velocity.x;
        velocity.linvel.z = horizontal_velocity.z;

        // Jump
        if keyboard.just_pressed(KeyCode::Space) && velocity.linvel.y.abs() < 0.1 {
            velocity.linvel.y = 3.0;
        }
    }
}

fn camera_look(
    mut motion_evr: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_motion: ResMut<MouseMotion>,
    mut player_query: Query<&mut PlayerCamera, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    for ev in motion_evr.read() {
        mouse_motion.delta += ev.delta;
    }

    for mut camera_config in player_query.iter_mut() {
        camera_config.yaw -= mouse_motion.delta.x * camera_config.sensitivity;
        camera_config.pitch -= mouse_motion.delta.y * camera_config.sensitivity;
        camera_config.pitch = camera_config.pitch.clamp(-1.5, 1.5);

        // Apply rotation to camera entity (child of player)
        for mut transform in camera_query.iter_mut() {
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                camera_config.yaw,
                camera_config.pitch,
                0.0,
            );
        }
    }

    mouse_motion.delta = Vec2::ZERO;
}

fn toggle_flashlight(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut flashlight_query: Query<(&mut Flashlight, &mut SpotLight)>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        for (mut flashlight, mut light) in flashlight_query.iter_mut() {
            flashlight.enabled = !flashlight.enabled;
            let new_intensity = if flashlight.enabled {
                flashlight.intensity
            } else {
                0.0
            };
            light.intensity = new_intensity;
            info!("Flashlight toggled: enabled={}, intensity={}", flashlight.enabled, new_intensity);
        }
    }
}

fn update_flashlight_transform(
    _camera_query: Query<&Transform, (With<Player>, Without<Flashlight>)>,
    _flashlight_query: Query<&Transform, With<Flashlight>>,
) {
    // Flashlight is a child, so it inherits parent transform automatically
    // This system is here for future expansion if needed
}
