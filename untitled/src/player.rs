use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    constants::*,
    resources::*,
};

/// Handles player movement based on WASD input
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Dash), With<Player>>,
    time: Res<Time>,
) {
    for (mut velocity, mut dash) in query.iter_mut() {
        // Update dash timers
        dash.cooldown_timer.tick(time.delta());
        dash.dash_timer.tick(time.delta());
        dash.iframe_timer.tick(time.delta());

        // Check if dash just finished
        if dash.is_dashing && dash.dash_timer.just_finished() {
            dash.is_dashing = false;
        }

        // Check if iframe just finished
        if dash.is_invincible && dash.iframe_timer.just_finished() {
            dash.is_invincible = false;
        }

        // Handle dash input (Shift key)
        if keyboard_input.just_pressed(KeyCode::ShiftLeft) {
            let mut dash_direction = Vec2::ZERO;

            // Get dash direction from WASD input
            if keyboard_input.pressed(KeyCode::KeyW) {
                dash_direction.y += 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                dash_direction.y -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyA) {
                dash_direction.x -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                dash_direction.x += 1.0;
            }

            // If no movement keys pressed, dash forward (up)
            if dash_direction == Vec2::ZERO {
                dash_direction = Vec2::Y;
            }

            dash.start_dash(dash_direction);
        }

        // Apply movement based on dash state
        if dash.is_dashing {
            // During dash, use dash speed and direction
            velocity.linvel = dash.dash_direction * DASH_SPEED;
        } else {
            // Normal movement
            let mut direction = Vec2::ZERO;

            // Check WASD input
            if keyboard_input.pressed(KeyCode::KeyW) {
                direction.y += 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                direction.y -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyA) {
                direction.x -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                direction.x += 1.0;
            }

            // Normalize diagonal movement
            if direction != Vec2::ZERO {
                direction = direction.normalize();
            }

            // Apply velocity
            velocity.linvel = direction * PLAYER_SPEED;
        }
    }
}

/// Handles player shooting mechanics
pub fn shoot_projectiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<(&Transform, &Velocity), (With<Player>, Without<Camera>)>,
    mut fire_timer: ResMut<FireTimer>,
    time: Res<Time>,
) {
    // Update the fire rate timer
    fire_timer.timer.tick(time.delta());

    // Check if player is holding shoot button and fire timer is ready
    let is_shooting = keyboard_input.pressed(KeyCode::Space) || mouse_input.pressed(MouseButton::Left);

    if is_shooting && fire_timer.timer.finished() {
        if let Ok((player_transform, player_velocity)) = player_query.single() {
            let player_pos = player_transform.translation.truncate();

            // Get mouse position in world coordinates
            let mut shoot_direction = Vec2::Y; // Default upward direction

            if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), camera_q.single()) {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                        shoot_direction = (world_pos - player_pos).normalize();
                    }
                }
            }

            // Calculate spawn position on the edge of the player closest to the cursor
            let spawn_offset = shoot_direction * (PLAYER_RADIUS + PROJECTILE_SIZE * 2.0 + 5.0); // Larger gap to prevent collision with player
            let spawn_pos = player_pos + spawn_offset;

            // Calculate projectile velocity: base velocity + player momentum
            let projectile_velocity = (shoot_direction * PROJECTILE_SPEED) + player_velocity.linvel;

            // Spawn projectile
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(PROJECTILE_SIZE))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 0.0))), // Yellow projectile
                Transform::from_translation(spawn_pos.extend(0.1)),
                Projectile {
                    lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                    team: Team::Player,
                },
                RigidBody::Dynamic,
                Collider::ball(PROJECTILE_SIZE * 1.2), // Slightly larger collision detection
                Sensor, // Make projectile a sensor so it doesn't physically interact with other projectiles
                Velocity::linear(projectile_velocity),
                // Enable collision events
                ActiveEvents::COLLISION_EVENTS,
            ));

            // Reset the fire timer for the next shot
            fire_timer.timer.reset();
        }
    }
}

/// Camera system that follows the player with cursor bias
pub fn camera_follow(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    windows: Query<&Window>,
    time: Res<Time>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) =
        (camera_query.single_mut(), player_query.single()) {

        let player_pos = player_transform.translation.truncate();
        let mut target_pos = player_pos;

        // Add cursor bias to camera position
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                // Convert cursor position to normalized coordinates (-1 to 1)
                let window_size = Vec2::new(window.width(), window.height());
                let mut cursor_normalized = (cursor_pos - window_size / 2.0) / (window_size / 2.0);

                // Flip Y axis to match world coordinates (screen Y goes down, world Y goes up)
                cursor_normalized.y = -cursor_normalized.y;

                // Apply cursor bias
                let cursor_bias = cursor_normalized * CURSOR_BIAS_DISTANCE * CURSOR_BIAS_STRENGTH;
                target_pos += cursor_bias;
            }
        }

        // Smoothly move camera towards target position
        let current_pos = camera_transform.translation.truncate();
        let direction = target_pos - current_pos;
        let move_distance = direction.length() * CAMERA_FOLLOW_SPEED * time.delta().as_secs_f32();

        if direction.length() > 0.1 {
            let new_pos = current_pos + direction.normalize() * move_distance;
            camera_transform.translation = new_pos.extend(camera_transform.translation.z);
        }
    }
}
