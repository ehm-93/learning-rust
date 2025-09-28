use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Custom event for projectile impacts
#[derive(Event)]
struct ProjectileImpactEvent {
    projectile: Entity,
    target: Entity,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Team {
    Player,
    Enemy,
    Neutral,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Projectile {
    lifetime: Timer,
    team: Team,
}

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Boundary;

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

impl Health {
    fn new(max_health: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
        }
    }
    
    fn take_damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
    }
    
    fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

// Custom event for damage dealing
#[derive(Event)]
struct DamageEvent {
    target: Entity,
    damage: f32,
}

// Detect projectile collisions and emit custom events
fn detect_projectile_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut impact_events: EventWriter<ProjectileImpactEvent>,
    projectiles: Query<&Projectile>,
    players: Query<&Player>,
    enemies: Query<&Enemy>,
    obstacles: Query<&Obstacle>,
    boundaries: Query<&Boundary>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check if either entity is a projectile
            let projectile_and_other = if projectiles.contains(*entity1) {
                Some((*entity1, *entity2))
            } else if projectiles.contains(*entity2) {
                Some((*entity2, *entity1))
            } else {
                None
            };

            if let Some((projectile, target)) = projectile_and_other {
                // Don't emit events for player-projectile collisions (friendly fire prevention)
                if players.contains(target) {
                    continue;
                }

                // Emit impact event for obstacles, boundaries, and enemies
                if obstacles.contains(target) || boundaries.contains(target) || enemies.contains(target) {
                    impact_events.write(ProjectileImpactEvent {
                        projectile,
                        target,
                    });
                }
            }
        }
    }
}

// Handle projectile impact events
fn handle_projectile_impacts(
    mut impact_events: EventReader<ProjectileImpactEvent>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
    projectile_query: Query<&Transform, With<Projectile>>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Projectile>)>,
    mut enemy_velocities: Query<&mut Velocity, With<Enemy>>,
) {
    for impact in impact_events.read() {
        // Get projectile and target positions for knockback calculation
        let knockback_direction = if let (Ok(projectile_transform), Ok(target_transform)) = 
            (projectile_query.get(impact.projectile), enemy_query.get(impact.target)) {
            // Calculate knockback direction from projectile to target
            let direction = (target_transform.translation.truncate() - 
                           projectile_transform.translation.truncate()).normalize_or_zero();
            Some(direction)
        } else {
            None
        };

        // Apply knockback to enemy if this was a projectile-enemy collision
        if let Some(direction) = knockback_direction {
            if let Ok(mut enemy_velocity) = enemy_velocities.get_mut(impact.target) {
                // Apply knockback impulse
                enemy_velocity.linvel += direction * KNOCKBACK_FORCE;
            }
            
            // Deal damage to enemy
            damage_events.write(DamageEvent {
                target: impact.target,
                damage: PROJECTILE_DAMAGE,
            });
        }

        // Clean up the projectile
        if let Ok(mut entity) = commands.get_entity(impact.projectile) {
            entity.despawn();
        }
    }
}#[derive(Resource)]
struct FireTimer {
    timer: Timer,
}

#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}

const PLAYER_SPEED: f32 = 200.0;
const PLAYER_RADIUS: f32 = 10.0;
const PROJECTILE_SPEED: f32 = 800.0;
const PROJECTILE_SIZE: f32 = 3.0;
const PROJECTILE_LIFETIME: f32 = 3.0;
const FIRE_RATE: f32 = 0.2; // 5 shots per second
const OBSTACLE_WIDTH: f32 = 40.0;
const OBSTACLE_HEIGHT: f32 = 80.0;
const CAMERA_FOLLOW_SPEED: f32 = 5.0; // How fast camera follows player
const CURSOR_BIAS_STRENGTH: f32 = 1.0; // How much cursor position affects camera
const CURSOR_BIAS_DISTANCE: f32 = 150.0; // Maximum distance cursor can bias camera
const MAP_WIDTH: f32 = 1200.0; // Total map width
const MAP_HEIGHT: f32 = 900.0; // Total map height
const WALL_THICKNESS: f32 = 20.0; // Thickness of boundary walls
const ENEMY_RADIUS: f32 = 8.0; // Enemy size
const ENEMY_SPEED: f32 = 120.0; // Enemy movement speed
const ENEMY_SPAWN_RATE: f32 = 2.0; // Seconds between enemy spawns
const MAX_ENEMIES: usize = 8; // Maximum enemies on screen
const KNOCKBACK_FORCE: f32 = 200.0; // Knockback impulse strength
const PLAYER_MAX_HEALTH: f32 = 100.0; // Player health
const ENEMY_MAX_HEALTH: f32 = 20.0; // Enemy health
const PROJECTILE_DAMAGE: f32 = 10.0; // Damage per projectile hit
const ENEMY_CONTACT_DAMAGE: f32 = 25.0; // Damage when enemy touches player

// Team relationship system
fn can_teams_damage(attacker: Team, target: Team) -> bool {
    match (attacker, target) {
        (Team::Player, Team::Enemy) => true,
        (Team::Enemy, Team::Player) => true,
        (Team::Player, Team::Player) => false,
        (Team::Enemy, Team::Enemy) => false,
        (_, Team::Neutral) => false,
        (Team::Neutral, _) => false,
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Combat Sandbox - Player Movement".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_event::<ProjectileImpactEvent>()
        .add_event::<DamageEvent>()
        .insert_resource(FireTimer {
            timer: Timer::from_seconds(FIRE_RATE, TimerMode::Repeating),
        })
        .insert_resource(EnemySpawnTimer {
            timer: Timer::from_seconds(ENEMY_SPAWN_RATE, TimerMode::Repeating),
        })
        .add_systems(Startup, (disable_gravity, setup))
        .add_systems(Update, (
            player_movement,
            shoot_projectiles,
            spawn_enemies,
            enemy_ai,
            cleanup_projectiles,
            camera_follow,
            detect_projectile_collisions,
            handle_projectile_impacts,
            process_damage,
            cleanup_dead_entities,
            detect_enemy_player_collisions,
        ))
        .run();
}

fn disable_gravity(mut query: Query<&mut RapierConfiguration>) {
    for mut config in &mut query {
        config.gravity = Vec2::ZERO;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn 2D camera with following component
    commands.spawn((
        Camera2d,
        MainCamera,
    ));

    // Spawn player as a white circle
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Player,
        Team::Player,
        Health::new(PLAYER_MAX_HEALTH),
        RigidBody::Dynamic,
        Collider::ball(PLAYER_RADIUS),
        // Lock rotation so the player doesn't spin
        LockedAxes::ROTATION_LOCKED,
        // Add Velocity component for movement
        Velocity::zero(),
        // Enable collision events for damage detection
        ActiveEvents::COLLISION_EVENTS,
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
            RigidBody::Fixed,
            Collider::cuboid(OBSTACLE_WIDTH / 2.0, OBSTACLE_HEIGHT / 2.0),
        ));
    }

    // Spawn boundary walls (invisible but solid)
    let half_width = MAP_WIDTH / 2.0;
    let half_height = MAP_HEIGHT / 2.0;
    let half_thickness = WALL_THICKNESS / 2.0;

    // Top wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(MAP_WIDTH + WALL_THICKNESS, WALL_THICKNESS))),
        MeshMaterial2d(materials.add(Color::srgba(0.2, 0.2, 0.2, 0.3))), // Semi-transparent dark gray
        Transform::from_translation(Vec3::new(0.0, half_height + half_thickness, -0.1)),
        Boundary,
        RigidBody::Fixed,
        Collider::cuboid((MAP_WIDTH + WALL_THICKNESS) / 2.0, WALL_THICKNESS / 2.0),
    ));

    // Bottom wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(MAP_WIDTH + WALL_THICKNESS, WALL_THICKNESS))),
        MeshMaterial2d(materials.add(Color::srgba(0.2, 0.2, 0.2, 0.3))),
        Transform::from_translation(Vec3::new(0.0, -half_height - half_thickness, -0.1)),
        Boundary,
        RigidBody::Fixed,
        Collider::cuboid((MAP_WIDTH + WALL_THICKNESS) / 2.0, WALL_THICKNESS / 2.0),
    ));

    // Left wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, MAP_HEIGHT))),
        MeshMaterial2d(materials.add(Color::srgba(0.2, 0.2, 0.2, 0.3))),
        Transform::from_translation(Vec3::new(-half_width - half_thickness, 0.0, -0.1)),
        Boundary,
        RigidBody::Fixed,
        Collider::cuboid(WALL_THICKNESS / 2.0, MAP_HEIGHT / 2.0),
    ));

    // Right wall
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, MAP_HEIGHT))),
        MeshMaterial2d(materials.add(Color::srgba(0.2, 0.2, 0.2, 0.3))),
        Transform::from_translation(Vec3::new(half_width + half_thickness, 0.0, -0.1)),
        Boundary,
        RigidBody::Fixed,
        Collider::cuboid(WALL_THICKNESS / 2.0, MAP_HEIGHT / 2.0),
    ));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
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
        velocity.linvel = direction * PLAYER_SPEED;
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
                Velocity::linear(projectile_velocity),
                // Enable collision events
                ActiveEvents::COLLISION_EVENTS,
            ));

            // Reset the fire timer for the next shot
            fire_timer.timer.reset();
        }
    }
}



fn camera_follow(
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

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
    enemies: Query<&Enemy>,
    player_query: Query<&Transform, With<Player>>,
) {
    // Update the spawn timer
    spawn_timer.timer.tick(time.delta());

    // Only spawn if timer is ready and we haven't hit the enemy limit
    if spawn_timer.timer.finished() && enemies.iter().count() < MAX_ENEMIES {
        if let Ok(player_transform) = player_query.single() {
            let player_pos = player_transform.translation.truncate();

            // Choose a random spawn position around the map edges, away from player
            let spawn_pos = get_enemy_spawn_position(player_pos);

            // Spawn enemy as a red circle
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(ENEMY_RADIUS))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.3, 0.3))), // Red enemy
                Transform::from_translation(spawn_pos.extend(0.0)),
                Enemy,
                Team::Enemy,
                Health::new(ENEMY_MAX_HEALTH),
                RigidBody::Dynamic,
                Collider::ball(ENEMY_RADIUS),
                LockedAxes::ROTATION_LOCKED,
                Velocity::zero(),
                // Enable collision events for damage detection
                ActiveEvents::COLLISION_EVENTS,
            ));

            // Reset the spawn timer
            spawn_timer.timer.reset();
        }
    }
}

fn get_enemy_spawn_position(player_pos: Vec2) -> Vec2 {
    use std::f32::consts::PI;

    // Spawn enemies around the map edges, at least 200 units away from player
    let min_distance = 200.0;
    let max_attempts = 10;

    for _ in 0..max_attempts {
        // Generate random angle
        let angle = fastrand::f32() * 2.0 * PI;

        // Choose distance from map center (spawn near edges)
        let spawn_distance = 300.0 + fastrand::f32() * 200.0; // Between 300-500 units from center

        let spawn_pos = Vec2::new(
            angle.cos() * spawn_distance,
            angle.sin() * spawn_distance,
        );

        // Check if spawn position is far enough from player
        if spawn_pos.distance(player_pos) >= min_distance {
            // Make sure it's within map bounds (with some margin)
            let half_width = MAP_WIDTH / 2.0 - 50.0;
            let half_height = MAP_HEIGHT / 2.0 - 50.0;

            if spawn_pos.x.abs() <= half_width && spawn_pos.y.abs() <= half_height {
                return spawn_pos;
            }
        }
    }

    // Fallback: spawn at a fixed position if random attempts fail
    Vec2::new(400.0, 300.0)
}

fn enemy_ai(
    mut enemy_query: Query<(&Transform, &mut Velocity), (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation.truncate();

        for (enemy_transform, mut enemy_velocity) in enemy_query.iter_mut() {
            let enemy_pos = enemy_transform.translation.truncate();

            // Calculate direction to player
            let direction_to_player = (player_pos - enemy_pos).normalize_or_zero();

            // Simple chase behavior - move toward player
            let desired_velocity = direction_to_player * ENEMY_SPEED;

            // Apply some smoothing to the velocity change for better movement feel
            let velocity_change = (desired_velocity - enemy_velocity.linvel) * 5.0 * time.delta().as_secs_f32();
            enemy_velocity.linvel += velocity_change;
        }
    }
}

fn detect_enemy_player_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut damage_events: EventWriter<DamageEvent>,
    players: Query<Entity, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check if collision is between enemy and player
            let collision_pair = if enemies.contains(*entity1) && players.contains(*entity2) {
                Some((*entity1, *entity2))
            } else if enemies.contains(*entity2) && players.contains(*entity1) {
                Some((*entity2, *entity1))
            } else {
                None
            };

            if let Some((_enemy, player)) = collision_pair {
                // Deal damage to player
                damage_events.write(DamageEvent {
                    target: player,
                    damage: ENEMY_CONTACT_DAMAGE,
                });
            }
        }
    }
}

fn process_damage(
    mut damage_events: EventReader<DamageEvent>,
    mut health_query: Query<&mut Health>,
) {
    for damage_event in damage_events.read() {
        if let Ok(mut health) = health_query.get_mut(damage_event.target) {
            health.take_damage(damage_event.damage);
        }
    }
}

fn cleanup_dead_entities(
    mut commands: Commands,
    health_query: Query<(Entity, &Health)>,
) {
    for (entity, health) in health_query.iter() {
        if health.is_dead() {
            commands.entity(entity).despawn();
        }
    }
}
