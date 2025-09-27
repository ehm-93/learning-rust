use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

// Core entity-team framework components
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Player,
    Enemy,
    Neutral,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
    }
}

#[derive(Component)]
pub struct Player {
    pub shoot_cooldown: Timer,
    pub dash_cooldown: Timer,
    pub invincibility_timer: Timer,
    pub is_dashing: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            shoot_cooldown: Timer::from_seconds(0.1, TimerMode::Once), // 10 shots per second max
            dash_cooldown: Timer::from_seconds(1.0, TimerMode::Once),  // Dash every second
            invincibility_timer: Timer::from_seconds(0.2, TimerMode::Once), // 0.2s invincibility
            is_dashing: false,
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub archetype: EnemyArchetype,
    pub attack_cooldown: Timer,
}

#[derive(Clone, Copy)]
pub enum EnemyArchetype {
    Chaser,    // Melee - chases player directly
    Shooter,   // Ranged - shoots at player from distance
    Shotgun,   // Shotgun spread pattern
}

impl Enemy {
    pub fn new(archetype: EnemyArchetype) -> Self {
        let cooldown_time = match archetype {
            EnemyArchetype::Chaser => 1.0,    // Attack every second when in melee range
            EnemyArchetype::Shooter => 1.5,   // Shoot every 1.5 seconds
            EnemyArchetype::Shotgun => 2.0,   // Slower shotgun firing
        };

        Self {
            archetype,
            attack_cooldown: Timer::from_seconds(cooldown_time, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub team: Team,
    pub lifetime: Timer,
}

impl Projectile {
    pub fn new(damage: f32, team: Team, lifetime_secs: f32) -> Self {
        Self {
            damage,
            team,
            lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct Obstacle;

// Game state resource
#[derive(Resource)]
pub struct GameState {
    pub team_relations: [[bool; 3]; 3], // [attacker][target] = can_damage
}

impl Default for GameState {
    fn default() -> Self {
        let mut relations = [[false; 3]; 3];

        // Players can damage enemies
        relations[Team::Player as usize][Team::Enemy as usize] = true;
        // Enemies can damage players
        relations[Team::Enemy as usize][Team::Player as usize] = true;
        // Nobody damages neutral
        // Neutral doesn't damage anyone

        Self {
            team_relations: relations,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<GameState>()
        .add_systems(Startup, setup_world)
        .add_systems(Update, (
            player_movement,
            player_dash,
            update_player_timers,
            player_shooting,
            update_enemy_timers,
            advanced_enemy_ai,
            enemy_shooting,
            update_projectiles,
            handle_damage_collisions,
            cleanup_dead_entities,
        ))
        .run();
}

fn setup_world(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Camera
    commands.spawn(Camera2d::default());

    // Create room boundaries (800x600 room)
    let wall_thickness = 20.0;
    let room_width = 800.0;
    let room_height = 600.0;

    // Wall material
    let wall_material = materials.add(Color::srgb(0.3, 0.3, 0.3));
    let wall_mesh = meshes.add(Rectangle::default());

    // Top wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(room_width / 2.0, wall_thickness / 2.0),
        Transform::from_xyz(0.0, room_height / 2.0, 0.0).with_scale(Vec3::new(room_width, wall_thickness, 1.0)),
        Obstacle,
        Mesh2d(wall_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
    ));

    // Bottom wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(room_width / 2.0, wall_thickness / 2.0),
        Transform::from_xyz(0.0, -room_height / 2.0, 0.0).with_scale(Vec3::new(room_width, wall_thickness, 1.0)),
        Obstacle,
        Mesh2d(wall_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
    ));

    // Left wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, room_height / 2.0),
        Transform::from_xyz(-room_width / 2.0, 0.0, 0.0).with_scale(Vec3::new(wall_thickness, room_height, 1.0)),
        Obstacle,
        Mesh2d(wall_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
    ));

    // Right wall
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, room_height / 2.0),
        Transform::from_xyz(room_width / 2.0, 0.0, 0.0).with_scale(Vec3::new(wall_thickness, room_height, 1.0)),
        Obstacle,
        Mesh2d(wall_mesh.clone()),
        MeshMaterial2d(wall_material.clone()),
    ));

    // Add some tactical pillars
    let pillar_size = 60.0;
    let pillar_mesh = meshes.add(Rectangle::default());
    let pillar_material = materials.add(Color::srgb(0.4, 0.4, 0.4));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(30.0, 30.0),
        Transform::from_xyz(-200.0, 100.0, 0.0).with_scale(Vec3::new(pillar_size, pillar_size, 1.0)),
        Obstacle,
        Mesh2d(pillar_mesh.clone()),
        MeshMaterial2d(pillar_material.clone()),
    ));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(30.0, 30.0),
        Transform::from_xyz(200.0, -100.0, 0.0).with_scale(Vec3::new(pillar_size, pillar_size, 1.0)),
        Obstacle,
        Mesh2d(pillar_mesh.clone()),
        MeshMaterial2d(pillar_material.clone()),
    ));

    // Create player visuals
    let player_mesh = meshes.add(Circle::new(15.0));
    let player_material = materials.add(Color::srgb(0.0, 0.8, 0.0));

    // Spawn player
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(15.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity::zero(),
        Health::new(100.0),
        Team::Player,
        Player::default(),
        LockedAxes::ROTATION_LOCKED,
        Damping { linear_damping: 0.0, angular_damping: 0.0 },
        Mesh2d(player_mesh),
        MeshMaterial2d(player_material),
    ));

    // Spawn different enemy archetypes
    let mut rng = rand::rng();
    let archetypes = [EnemyArchetype::Chaser, EnemyArchetype::Shooter, EnemyArchetype::Shotgun];

    for i in 0..3 {
        let x = rng.random_range(-300.0..300.0);
        let y = rng.random_range(-200.0..200.0);
        let archetype = archetypes[i % archetypes.len()];

        // Different visual styles for different archetypes
        let (color, size) = match archetype {
            EnemyArchetype::Chaser => (Color::srgb(0.8, 0.0, 0.0), 12.0),   // Red, normal size
            EnemyArchetype::Shooter => (Color::srgb(0.6, 0.0, 0.6), 10.0),  // Purple, smaller
            EnemyArchetype::Shotgun => (Color::srgb(0.8, 0.4, 0.0), 14.0),  // Orange, larger
        };

        let enemy_archetype_mesh = meshes.add(Circle::new(size));
        let enemy_archetype_material = materials.add(color);

        commands.spawn((
            RigidBody::Dynamic,
            Collider::ball(size),
            Transform::from_xyz(x, y, 0.0),
            Velocity::zero(),
            Health::new(50.0),
            Team::Enemy,
            Enemy::new(archetype),
            LockedAxes::ROTATION_LOCKED,
            Damping { linear_damping: 0.0, angular_damping: 0.0 },
            Mesh2d(enemy_archetype_mesh),
            MeshMaterial2d(enemy_archetype_material),
        ));
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Velocity, (With<Player>, Without<Projectile>)>,
) {
    for mut velocity in player_query.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction != Vec2::ZERO {
            direction = direction.normalize();
            let speed = 300.0;
            velocity.linvel = direction * speed;
        } else {
            // Stop the player when no keys are pressed
            velocity.linvel = Vec2::ZERO;
        }
    }
}

fn update_player_timers(
    mut player_query: Query<&mut Player>,
    time: Res<Time>,
) {
    for mut player in player_query.iter_mut() {
        player.shoot_cooldown.tick(time.delta());
        player.dash_cooldown.tick(time.delta());
        player.invincibility_timer.tick(time.delta());

        // End dash state when invincibility ends
        if player.is_dashing && player.invincibility_timer.finished() {
            player.is_dashing = false;
        }
    }
}

fn player_dash(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Velocity, &mut Player)>,
) {
    for (mut velocity, mut player) in player_query.iter_mut() {
        // Dash with Shift key
        if keyboard_input.just_pressed(KeyCode::ShiftLeft) && player.dash_cooldown.finished() {
            // Reset cooldowns
            player.dash_cooldown.reset();
            player.invincibility_timer.reset();
            player.is_dashing = true;

            // Get current movement direction or default to upward
            let dash_direction = if velocity.linvel.length() > 10.0 {
                velocity.linvel.normalize()
            } else {
                Vec2::Y
            };

            // Apply dash impulse
            let dash_speed = 600.0;
            velocity.linvel = dash_direction * dash_speed;
        }
    }
}

fn player_shooting(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Transform, &mut Player)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Primary fire - mouse or space
    let should_shoot = mouse_input.pressed(MouseButton::Left) ||
                       keyboard_input.pressed(KeyCode::Space);

    if should_shoot {
        if let (Ok((player_transform, mut player)), Ok((camera, camera_transform)), Ok(window)) =
            (player_query.single_mut(), camera_query.single(), windows.single()) {

            // Check cooldown
            if !player.shoot_cooldown.finished() {
                return;
            }

            // Reset cooldown
            player.shoot_cooldown.reset();

            let player_pos = player_transform.translation.truncate();

            // Get shooting direction from mouse position
            let shoot_direction = if let Some(cursor_pos) = window.cursor_position() {
                // Convert cursor position to world coordinates
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    (world_pos - player_pos).normalize()
                } else {
                    Vec2::Y // Default upward if conversion fails
                }
            } else {
                Vec2::Y // Default upward if no cursor
            };

            let bullet_speed = 500.0;
            let spawn_offset = shoot_direction * 20.0; // Spawn bullet ahead of player

            // Create bullet visuals
            let bullet_mesh = meshes.add(Circle::new(3.0));
            let bullet_material = materials.add(Color::srgb(1.0, 1.0, 0.0));

            commands.spawn((
                RigidBody::Dynamic,
                Collider::ball(3.0),
                Transform::from_xyz(
                    player_pos.x + spawn_offset.x,
                    player_pos.y + spawn_offset.y,
                    0.0
                ),
                Velocity {
                    linvel: shoot_direction * bullet_speed,
                    angvel: 0.0,
                },
                Projectile::new(25.0, Team::Player, 3.0),
                LockedAxes::ROTATION_LOCKED,
                Mesh2d(bullet_mesh),
                MeshMaterial2d(bullet_material),
            ));
        }
    }
}

fn update_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut projectile) in projectile_query.iter_mut() {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_enemy_timers(
    mut enemy_query: Query<&mut Enemy>,
    time: Res<Time>,
) {
    for mut enemy in enemy_query.iter_mut() {
        enemy.attack_cooldown.tick(time.delta());
    }
}

fn advanced_enemy_ai(
    mut enemy_query: Query<(&Transform, &mut Velocity, &Enemy), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (enemy_transform, mut enemy_velocity, enemy) in enemy_query.iter_mut() {
            let direction = (player_transform.translation.truncate() -
                           enemy_transform.translation.truncate()).normalize();
            let distance = (player_transform.translation.truncate() -
                          enemy_transform.translation.truncate()).length();

            let speed = match enemy.archetype {
                EnemyArchetype::Chaser => {
                    // Always chase at full speed
                    100.0
                },
                EnemyArchetype::Shooter => {
                    // Keep distance, move slower when in range
                    if distance > 200.0 {
                        60.0  // Move closer
                    } else if distance < 150.0 {
                        -40.0 // Back away
                    } else {
                        20.0  // Strafe slowly
                    }
                },
                EnemyArchetype::Shotgun => {
                    // Moderate approach, stops at medium range
                    if distance > 120.0 {
                        80.0
                    } else {
                        0.0
                    }
                }
            };

            if speed != 0.0 {
                enemy_velocity.linvel = direction * speed;
            } else {
                enemy_velocity.linvel = Vec2::ZERO;
            }
        }
    }
}

fn enemy_shooting(
    mut commands: Commands,
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (enemy_transform, mut enemy) in enemy_query.iter_mut() {
            // Only ranged enemies shoot
            match enemy.archetype {
                EnemyArchetype::Chaser => continue, // Melee only
                EnemyArchetype::Shooter | EnemyArchetype::Shotgun => {
                    if !enemy.attack_cooldown.finished() {
                        continue;
                    }

                    let distance = (player_transform.translation.truncate() -
                                  enemy_transform.translation.truncate()).length();

                    // Check if player is in range
                    let max_range = match enemy.archetype {
                        EnemyArchetype::Shooter => 250.0,
                        EnemyArchetype::Shotgun => 150.0,
                        _ => 0.0,
                    };

                    if distance <= max_range {
                        enemy.attack_cooldown.reset();

                        let direction = (player_transform.translation.truncate() -
                                       enemy_transform.translation.truncate()).normalize();

                        // Create enemy bullet visuals
                        let bullet_mesh = meshes.add(Circle::new(2.5));
                        let bullet_material = materials.add(Color::srgb(0.9, 0.1, 0.1));

                        match enemy.archetype {
                            EnemyArchetype::Shooter => {
                                // Single bullet
                                let bullet_speed = 300.0;
                                let spawn_offset = direction * 15.0;

                                commands.spawn((
                                    RigidBody::Dynamic,
                                    Collider::ball(2.5),
                                    Transform::from_xyz(
                                        enemy_transform.translation.x + spawn_offset.x,
                                        enemy_transform.translation.y + spawn_offset.y,
                                        0.0
                                    ),
                                    Velocity {
                                        linvel: direction * bullet_speed,
                                        angvel: 0.0,
                                    },
                                    Projectile::new(20.0, Team::Enemy, 4.0),
                                    LockedAxes::ROTATION_LOCKED,
                                    Damping { linear_damping: 0.0, angular_damping: 0.0 },
                                    Mesh2d(bullet_mesh),
                                    MeshMaterial2d(bullet_material),
                                ));
                            },
                            EnemyArchetype::Shotgun => {
                                // Spread pattern - 5 bullets
                                let bullet_speed = 250.0;
                                let spread_angle = 0.4; // radians

                                for i in 0..5 {
                                    let angle_offset = (i as f32 - 2.0) * spread_angle / 4.0;
                                    let spread_direction = Vec2::new(
                                        direction.x * angle_offset.cos() - direction.y * angle_offset.sin(),
                                        direction.x * angle_offset.sin() + direction.y * angle_offset.cos()
                                    );

                                    let spawn_offset = spread_direction * 15.0;
                                    let bullet_mesh_clone = meshes.add(Circle::new(2.0));
                                    let bullet_material_clone = materials.add(Color::srgb(0.9, 0.5, 0.1));

                                    commands.spawn((
                                        RigidBody::Dynamic,
                                        Collider::ball(2.0),
                                        Transform::from_xyz(
                                            enemy_transform.translation.x + spawn_offset.x,
                                            enemy_transform.translation.y + spawn_offset.y,
                                            0.0
                                        ),
                                        Velocity {
                                            linvel: spread_direction * bullet_speed,
                                            angvel: 0.0,
                                        },
                                        Projectile::new(15.0, Team::Enemy, 3.0),
                                        LockedAxes::ROTATION_LOCKED,
                                        Damping { linear_damping: 0.0, angular_damping: 0.0 },
                                        Mesh2d(bullet_mesh_clone),
                                        MeshMaterial2d(bullet_material_clone),
                                    ));
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn handle_damage_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut health_query: Query<(&mut Health, &Team)>,
    player_query: Query<&Player>,
    projectile_query: Query<&Projectile>,
    game_state: Res<GameState>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check if one is a projectile and the other has health
            let (projectile_entity, target_entity) =
                if projectile_query.contains(*entity1) && health_query.contains(*entity2) {
                    (*entity1, *entity2)
                } else if projectile_query.contains(*entity2) && health_query.contains(*entity1) {
                    (*entity2, *entity1)
                } else {
                    continue;
                };

            if let (Ok(projectile), Ok((mut health, target_team))) =
                (projectile_query.get(projectile_entity), health_query.get_mut(target_entity)) {

                // Check if target is a player with invincibility frames
                let is_invincible = if let Ok(player) = player_query.get(target_entity) {
                    player.is_dashing && !player.invincibility_timer.finished()
                } else {
                    false
                };

                // Check if this team can damage the target team and target is not invincible
                if !is_invincible && game_state.team_relations[projectile.team as usize][*target_team as usize] {
                    health.take_damage(projectile.damage);
                    commands.entity(projectile_entity).despawn();
                }
            }
        }
    }
}

fn cleanup_dead_entities(
    mut commands: Commands,
    health_query: Query<(Entity, &Health)>,
) {
    for (entity, health) in health_query.iter() {
        if !health.is_alive() {
            commands.entity(entity).despawn();
        }
    }
}
