use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MouseMotion>()
            .add_systems(OnEnter(GameState::InGame), setup_player)
            .add_systems(
                Update,
                (
                    player_movement,
                    apply_movement,
                    camera_look,
                    toggle_flashlight,
                    update_flashlight_transform,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn setup_player(mut commands: Commands) {
    // Player camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.7, 0.0),
        PlayerCamera {
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        },
        PlayerMovement {
            speed: 5.0,
            velocity: Vec3::ZERO,
        },
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

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerMovement, &PlayerCamera), With<Player>>,
) {
    for (mut movement, camera) in query.iter_mut() {
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

        movement.velocity = if direction.length() > 0.0 {
            direction.normalize() * movement.speed
        } else {
            Vec3::ZERO
        };
    }
}

fn apply_movement(
    mut query: Query<(&PlayerMovement, &mut Transform), With<Player>>,
    time: Res<Time>,
) {
    for (movement, mut transform) in query.iter_mut() {
        transform.translation += movement.velocity * time.delta_secs();
    }
}

fn camera_look(
    mut motion_evr: MessageReader<bevy::input::mouse::MouseMotion>,
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
