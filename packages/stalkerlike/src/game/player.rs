use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

use super::components::*;
use super::resources::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MouseMotion>()
            .add_systems(OnEnter(GameState::InGame), (setup_player, cursor_grab))
            .add_systems(OnExit(GameState::InGame), cursor_release)
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

fn setup_player(mut commands: Commands) {
    // Player with physics-based character controller
    commands.spawn((
        Player,
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.7, 0.0),
        PlayerCamera {
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        },
        RigidBody::Dynamic,
        Collider::capsule_y(0.5, 0.4),
        Velocity::default(),
        LockedAxes::ROTATION_LOCKED, // Prevent player from tipping over
    ))
    .with_children(|parent| {
        // Spawn flashlight as child
        parent.spawn((
            Flashlight::default(),
            SpotLight {
                intensity: 0.0,
                range: 20.0,
                radius: 0.5,
                shadows_enabled: true,
                outer_angle: 0.8,
                inner_angle: 0.6,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
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
        velocity.linvel = if direction.length() > 0.0 {
            direction.normalize() * speed
        } else {
            Vec3::ZERO
        };
    }
}

fn camera_look(
    mut motion_evr: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_motion: ResMut<MouseMotion>,
    mut query: Query<(&mut PlayerCamera, &mut Transform), With<Player>>,
) {
    for ev in motion_evr.read() {
        mouse_motion.delta += ev.delta;
    }

    for (mut camera, mut transform) in query.iter_mut() {
        camera.yaw -= mouse_motion.delta.x * camera.sensitivity;
        camera.pitch -= mouse_motion.delta.y * camera.sensitivity;
        camera.pitch = camera.pitch.clamp(-1.5, 1.5);

        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            camera.yaw,
            camera.pitch,
            0.0,
        );
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
            light.intensity = if flashlight.enabled {
                flashlight.intensity
            } else {
                0.0
            };
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
