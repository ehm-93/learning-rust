use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    constants::*,
    sounds::*,
    player::resources::*,
};

// Add missing constant that was used in player shooting
const PROJECTILE_MOMENTUM_TRANSFER: f32 = 0.5;
use super::{Player, Dash, GrenadeThrower, FireTimer, PlayerConfig};
use super::actions::{PlayerActionEvent, PlayerAction};

/// Handles player movement based on player action events
pub fn player_movement(
    mut action_events: EventReader<PlayerActionEvent>,
    mut query: Query<(&mut Velocity, &mut Dash), With<Player>>,
    time: Res<Time>,
    config: Res<PlayerConfig>,
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

        // Process action events
        let mut movement = Vec2::ZERO;
        let mut dash_requested = false;
        let mut dash_direction = Vec2::ZERO;

        for action_event in action_events.read() {
            match action_event.action {
                PlayerAction::MoveUp if action_event.is_active() => {
                    movement.y += action_event.value;
                    dash_direction.y += action_event.value;
                }
                PlayerAction::MoveDown if action_event.is_active() => {
                    movement.y -= action_event.value;
                    dash_direction.y -= action_event.value;
                }
                PlayerAction::MoveLeft if action_event.is_active() => {
                    movement.x -= action_event.value;
                    dash_direction.x -= action_event.value;
                }
                PlayerAction::MoveRight if action_event.is_active() => {
                    movement.x += action_event.value;
                    dash_direction.x += action_event.value;
                }
                PlayerAction::Dash if action_event.just_started() => {
                    dash_requested = true;
                }
                _ => {}
            }
        }

        // Handle dash
        if dash_requested && dash_direction != Vec2::ZERO {
            dash.start_dash(dash_direction.normalize());
        }

        // Apply movement
        if dash.is_dashing {
            velocity.linvel = dash.dash_direction * DASH_SPEED;
        } else {
            // Normalize movement to prevent faster diagonal movement
            if movement != Vec2::ZERO {
                movement = movement.normalize();
            }
            velocity.linvel = movement * PLAYER_SPEED * config.movement_speed_multiplier;
        }
    }
}

/// Handles player shooting mechanics
pub fn shoot_projectiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_events: EventReader<PlayerActionEvent>,
    player_query: Query<(&Transform, &Velocity), (With<Player>, Without<Camera>)>,
    mut fire_timer: ResMut<FireTimer>,
    game_sounds: Res<GameSounds>,
    time: Res<Time>,
) {
    // Update fire timer
    fire_timer.timer.tick(time.delta());

    // Process shoot action events
    for action_event in action_events.read() {
        if matches!(action_event.action, PlayerAction::Shoot) &&
           (action_event.just_started() || action_event.is_active()) &&
           fire_timer.timer.finished() {

            if let Ok((player_transform, player_velocity)) = player_query.single() {
                let player_pos = player_transform.translation.truncate();

                // Use world position from action event if available, otherwise default upward
                let shoot_direction = if let Some(target_pos) = action_event.world_position {
                    (target_pos - player_pos).normalize()
                } else {
                    Vec2::Y
                };

                // Calculate spawn position on the edge of the player closest to the target
                let spawn_offset = shoot_direction * (PLAYER_RADIUS + PROJECTILE_SIZE * 2.0 + 5.0);
                let spawn_pos = player_pos + spawn_offset;

                // Calculate projectile velocity: base velocity + player momentum
                let projectile_velocity = (shoot_direction * PROJECTILE_SPEED) +
                    (player_velocity.linvel * PROJECTILE_MOMENTUM_TRANSFER);

                // Spawn projectile
                commands.spawn((
                    Mesh2d(meshes.add(Circle::new(PROJECTILE_SIZE))),
                    MeshMaterial2d(materials.add(Color::WHITE)),
                    Transform::from_translation(spawn_pos.extend(0.1)),
                    Projectile {
                        lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                        team: Team::Player,
                    },
                    RigidBody::Dynamic,
                    Collider::ball(PROJECTILE_SIZE),
                    Velocity::linear(projectile_velocity),
                    ActiveEvents::COLLISION_EVENTS,
                ));

                // Play shooting sound
                commands.spawn((
                    AudioPlayer(game_sounds.gun_01.clone()),
                    PlaybackSettings::DESPAWN,
                ));

                // Reset fire timer
                fire_timer.timer.reset();
            }
        }
    }
}

/// Handles player grenade throwing mechanics
pub fn throw_grenades(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_events: EventReader<PlayerActionEvent>,
    mut player_query: Query<(&Transform, &Velocity, &mut GrenadeThrower), (With<Player>, Without<Camera>)>,
    time: Res<Time>,
) {
    // Update grenade cooldown timer
    for (_, _, mut grenade_thrower) in player_query.iter_mut() {
        grenade_thrower.cooldown_timer.tick(time.delta());
    }

    // Process grenade throw action events
    for action_event in action_events.read() {
        if matches!(action_event.action, PlayerAction::ThrowGrenade) &&
           (action_event.just_started() || action_event.is_active()) {
            if let Ok((player_transform, player_velocity, mut grenade_thrower)) = player_query.single_mut() {
                if grenade_thrower.can_throw() {
                    let player_pos = player_transform.translation.truncate();

                    // Use world position from action event if available, otherwise default upward
                    let throw_direction = if let Some(target_pos) = action_event.world_position {
                        (target_pos - player_pos).normalize()
                    } else {
                        Vec2::Y
                    };

                    // Calculate spawn position on the edge of the player
                    let spawn_offset = throw_direction * (PLAYER_RADIUS + GRENADE_SIZE * 2.0 + 5.0);
                    let spawn_pos = player_pos + spawn_offset;

                    // Calculate grenade velocity: base velocity + player momentum (reduced)
                    let grenade_velocity = (throw_direction * GRENADE_SPEED) + (player_velocity.linvel * 0.3);

                    // Spawn grenade
                    commands.spawn((
                        Mesh2d(meshes.add(Circle::new(GRENADE_SIZE))),
                        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.8, 0.2))), // Green grenade
                        Transform::from_translation(spawn_pos.extend(0.1)),
                        Grenade {
                            fuse_timer: Timer::from_seconds(GRENADE_FUSE_TIME, TimerMode::Once),
                            team: Team::Player,
                        },
                        RigidBody::Dynamic,
                        Collider::ball(GRENADE_SIZE),
                        Restitution::coefficient(GRENADE_BOUNCE), // Make it bouncy
                        Velocity::linear(grenade_velocity),
                        ActiveEvents::COLLISION_EVENTS,
                        Damping {
                            linear_damping: GRENADE_DAMPING,
                            angular_damping: 0.0,
                        },
                    ));

                    // Reset grenade cooldown
                    grenade_thrower.throw_grenade();
                }
            }
        }
    }
}

/// Camera system that follows the player with cursor bias
pub fn camera_follow(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    windows: Query<&Window>,
    _time: Res<Time>,
    config: Res<PlayerConfig>,
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

                // Apply cursor bias with configurable strength and max distance
                let cursor_bias = cursor_normalized * config.cursor_bias_strength * config.cursor_bias_max_distance;
                target_pos += cursor_bias;
            }
        }

        // Smoothly move camera towards target position
        let current_pos = camera_transform.translation.truncate();
        let new_pos = current_pos.lerp(target_pos, config.camera_smoothing);
        camera_transform.translation = new_pos.extend(camera_transform.translation.z);
    }
}

/// System to handle camera zoom events and apply zoom to all cameras
pub fn handle_camera_zoom(
    mut action_events: EventReader<PlayerActionEvent>,
    mut camera_zoom: ResMut<CameraZoom>,
    mut camera_query: Query<&mut Transform, With<crate::components::MainCamera>>,
) {
    let mut zoom_changed = false;

    // Handle zoom input events
    for event in action_events.read() {
        match event.action {
            PlayerAction::ZoomIn => {
                camera_zoom.level = (camera_zoom.level - camera_zoom.sensitivity).max(camera_zoom.min_zoom);
                zoom_changed = true;
            }
            PlayerAction::ZoomOut => {
                camera_zoom.level = (camera_zoom.level + camera_zoom.sensitivity).min(camera_zoom.max_zoom);
                zoom_changed = true;
            }
            _ => {}
        }
    }

    // Apply zoom level to all main cameras if zoom changed
    if zoom_changed {
        for mut camera_transform in camera_query.iter_mut() {
            camera_transform.scale = Vec3::splat(camera_zoom.level);
        }
    }
}
