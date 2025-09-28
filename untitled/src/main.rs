use bevy::prelude::*;
use avian2d::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Projectile {
    lifetime: Timer,
    bounces_remaining: u32,
}

#[derive(Component)]
struct Obstacle;

#[derive(Resource)]
struct FireTimer {
    timer: Timer,
}

const PLAYER_SPEED: f32 = 200.0;
const PLAYER_RADIUS: f32 = 10.0;
const PROJECTILE_SPEED: f32 = 800.0;
const PROJECTILE_SIZE: f32 = 3.0;
const PROJECTILE_LIFETIME: f32 = 3.0;
const FIRE_RATE: f32 = 0.1; // 10 shots per second
const OBSTACLE_WIDTH: f32 = 40.0;
const OBSTACLE_HEIGHT: f32 = 80.0;
const MAX_BOUNCES: u32 = 3;
const BOUNCE_RESTITUTION: f32 = 0.8; // How much velocity is retained after bounce
const PROJECTILE_FRICTION: f32 = 0.98; // Friction coefficient (98% speed retained per frame)
const MIN_PROJECTILE_SPEED: f32 = 150.0; // Minimum speed before despawning

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Combat Sandbox - Player Movement".to_string(),
                    resolution: (800.0, 600.0).into(),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
        ))
        .insert_resource(FireTimer {
            timer: Timer::from_seconds(FIRE_RATE, TimerMode::Repeating),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, shoot_projectiles, track_bounces, monitor_projectile_speed, cleanup_projectiles))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    // Spawn player as a white circle
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Player,
        RigidBody::Dynamic,
        Collider::circle(PLAYER_RADIUS),
        // Lock rotation so the player doesn't spin
        LockedAxes::ROTATION_LOCKED,
    ));

    // Spawn obstacles as gray rectangles with varied rotations
    let obstacle_data = [
        (Vec2::new(150.0, 100.0), 0.3),
        (Vec2::new(-150.0, -100.0), -0.7),
        (Vec2::new(200.0, -150.0), 0.5),
        (Vec2::new(-200.0, 150.0), -0.2),
        (Vec2::new(0.0, 200.0), 0.8),
        (Vec2::new(0.0, -200.0), -0.4),
    ];

    for (pos, rotation) in obstacle_data {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(OBSTACLE_WIDTH, OBSTACLE_HEIGHT))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))), // Gray obstacles
            Transform::from_translation(pos.extend(0.0)).with_rotation(Quat::from_rotation_z(rotation)),
            Obstacle,
            RigidBody::Static,
            Collider::rectangle(OBSTACLE_WIDTH, OBSTACLE_HEIGHT),
        ));
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut LinearVelocity, With<Player>>,
) {
    for mut velocity in query.iter_mut() {
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
        velocity.0 = direction * PLAYER_SPEED;
    }
}

fn shoot_projectiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<(&Transform, &LinearVelocity), (With<Player>, Without<Camera>)>,
    mut fire_timer: ResMut<FireTimer>,
    time: Res<Time>,
) {
    // Update the fire rate timer
    fire_timer.timer.tick(time.delta());

    // Check if player is holding shoot button and fire timer is ready
    let is_shooting = keyboard_input.pressed(KeyCode::Space) || mouse_input.pressed(MouseButton::Left);

    if is_shooting && fire_timer.timer.finished() {
        if let Ok((player_transform, player_velocity)) = player_query.get_single() {
            let player_pos = player_transform.translation.truncate();

            // Get mouse position in world coordinates
            let mut shoot_direction = Vec2::Y; // Default upward direction

            if let (Ok(window), Ok((camera, camera_transform))) = (windows.get_single(), camera_q.get_single()) {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                        shoot_direction = (world_pos - player_pos).normalize();
                    }
                }
            }

            // Calculate spawn position on the edge of the player closest to the cursor
            let spawn_offset = shoot_direction * (PLAYER_RADIUS + PROJECTILE_SIZE + 1.0); // Small gap to prevent collision
            let spawn_pos = player_pos + spawn_offset;

            // Calculate projectile velocity: base velocity + player momentum
            let projectile_velocity = (shoot_direction * PROJECTILE_SPEED) + player_velocity.0;

            // Spawn projectile
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(PROJECTILE_SIZE))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 0.0))), // Yellow projectile
                Transform::from_translation(spawn_pos.extend(0.1)),
                Projectile {
                    lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                    bounces_remaining: MAX_BOUNCES,
                },
                RigidBody::Dynamic,
                Collider::circle(PROJECTILE_SIZE),
                LinearVelocity(projectile_velocity),
                // Enable bouncing with restitution
                Restitution::new(BOUNCE_RESTITUTION),
                // Add gentle friction via linear damping
                LinearDamping(1.0 - PROJECTILE_FRICTION),
                // Disable gravity for projectiles
                GravityScale(0.0),
            ));

            // Reset the fire timer for the next shot
            fire_timer.timer.reset();
        }
    }
}

fn track_bounces(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    mut projectiles: Query<&mut Projectile>,
    obstacles: Query<&Obstacle>,
) {
    for CollisionStarted(entity1, entity2) in collision_events.read() {
        // Check if one entity is a projectile and the other is an obstacle
        let (projectile_entity, _obstacle_entity) =
            if projectiles.contains(*entity1) && obstacles.contains(*entity2) {
                (*entity1, *entity2)
            } else if projectiles.contains(*entity2) && obstacles.contains(*entity1) {
                (*entity2, *entity1)
            } else {
                continue; // Not a projectile-obstacle collision
            };

        if let Ok(mut projectile) = projectiles.get_mut(projectile_entity) {
            if projectile.bounces_remaining > 0 {
                projectile.bounces_remaining -= 1;
            } else {
                // No bounces left, despawn the projectile
                commands.entity(projectile_entity).despawn();
            }
        }
    }
}

fn monitor_projectile_speed(
    mut commands: Commands,
    projectiles: Query<(Entity, &LinearVelocity), With<Projectile>>,
) {
    for (entity, velocity) in projectiles.iter() {
        let speed = velocity.0.length();

        // Despawn projectiles that are moving too slowly
        if speed < MIN_PROJECTILE_SPEED {
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Projectile)>,
) {
    for (entity, mut projectile) in projectiles.iter_mut() {
        projectile.lifetime.tick(time.delta());

        if projectile.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
