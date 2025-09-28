use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::{
    components::*,
    constants::*,
    resources::*,
};

/// Configuration for enemy archetype properties
#[derive(Debug, Clone)]
pub struct ArchetypeConfig {
    pub health: f32,
    pub speed: f32,
    pub radius: f32,
    pub color: Color,
    pub preferred_distance: f32,
    pub fire_rate: f32,
}

impl ArchetypeConfig {
    /// Get configuration for a specific enemy archetype
    pub fn for_archetype(archetype: EnemyArchetype) -> Self {
        match archetype {
            EnemyArchetype::SmallMelee => Self {
                health: SMALL_MELEE_HEALTH,
                speed: SMALL_MELEE_SPEED,
                radius: SMALL_MELEE_RADIUS,
                color: Color::srgb(1.0, 0.2, 0.2), // Bright red
                preferred_distance: 0.0,
                fire_rate: 1.0,
            },
            EnemyArchetype::BigMelee => Self {
                health: BIG_MELEE_HEALTH,
                speed: BIG_MELEE_SPEED,
                radius: BIG_MELEE_RADIUS,
                color: Color::srgb(0.6, 0.1, 0.1), // Dark red
                preferred_distance: 0.0,
                fire_rate: 1.0,
            },
            EnemyArchetype::Shotgunner => Self {
                health: SHOTGUNNER_HEALTH,
                speed: SHOTGUNNER_SPEED,
                radius: SHOTGUNNER_RADIUS,
                color: Color::srgb(1.0, 0.5, 0.0), // Orange
                preferred_distance: SHOTGUNNER_RANGE,
                fire_rate: SHOTGUNNER_FIRE_RATE,
            },
            EnemyArchetype::Sniper => Self {
                health: SNIPER_HEALTH,
                speed: SNIPER_SPEED,
                radius: SNIPER_RADIUS,
                color: Color::srgb(0.0, 0.8, 0.2), // Green
                preferred_distance: SNIPER_RANGE,
                fire_rate: SNIPER_FIRE_RATE,
            },
            EnemyArchetype::MachineGunner => Self {
                health: MACHINE_GUNNER_HEALTH,
                speed: MACHINE_GUNNER_SPEED,
                radius: MACHINE_GUNNER_RADIUS,
                color: Color::srgb(0.8, 0.0, 0.8), // Purple
                preferred_distance: MACHINE_GUNNER_RANGE,
                fire_rate: MACHINE_GUNNER_FIRE_RATE,
            },
        }
    }
}

/// Behavior context for AI decision making
#[derive(Debug, Clone)]
pub struct BehaviorContext {
    pub enemy_pos: Vec2,
    pub distance_to_player: f32,
    pub direction_to_player: Vec2,
}

/// Enemy behavior implementations for each archetype
impl EnemyArchetype {
    /// Execute the AI behavior for this archetype
    pub fn execute_behavior(
        &self,
        context: &BehaviorContext,
        ai: &mut AIBehavior,
        velocity: &mut Velocity,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        laser_sight: Option<&mut LaserSight>,
    ) {
        match self {
            EnemyArchetype::SmallMelee => {
                self.small_melee_behavior(context, velocity);
            },
            EnemyArchetype::BigMelee => {
                self.big_melee_behavior(context, velocity);
            },
            EnemyArchetype::Shotgunner => {
                self.shotgunner_behavior(context, ai, velocity, commands, meshes, materials);
            },
            EnemyArchetype::Sniper => {
                self.sniper_behavior(context, ai, velocity, commands, meshes, materials, laser_sight);
            },
            EnemyArchetype::MachineGunner => {
                self.machine_gunner_behavior(context, ai, velocity, commands, meshes, materials);
            },
        }
    }

    /// Small melee enemy behavior - charge directly at player
    fn small_melee_behavior(&self, context: &BehaviorContext, velocity: &mut Velocity) {
        let config = ArchetypeConfig::for_archetype(*self);
        velocity.linvel = context.direction_to_player * config.speed;
    }

    /// Big melee enemy behavior - charge at player with higher health/damage
    fn big_melee_behavior(&self, context: &BehaviorContext, velocity: &mut Velocity) {
        let config = ArchetypeConfig::for_archetype(*self);
        velocity.linvel = context.direction_to_player * config.speed;
    }

    /// Shotgunner behavior - maintain distance and fire spreads
    fn shotgunner_behavior(
        &self,
        context: &BehaviorContext,
        ai: &mut AIBehavior,
        velocity: &mut Velocity,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        let config = ArchetypeConfig::for_archetype(*self);

        // Positioning behavior
        if context.distance_to_player > config.preferred_distance + 20.0 {
            // Too far, move closer
            velocity.linvel = context.direction_to_player * config.speed;
        } else if context.distance_to_player < config.preferred_distance - 20.0 {
            // Too close, back away
            velocity.linvel = -context.direction_to_player * config.speed;
        } else {
            // At good distance, strafe around player
            let perpendicular = Vec2::new(-context.direction_to_player.y, context.direction_to_player.x);
            velocity.linvel = perpendicular * config.speed * 0.5;
        }

        // Shooting behavior
        if ai.timer.finished() {
            spawn_shotgun_spread(commands, meshes, materials, context.enemy_pos, context.direction_to_player);
            ai.timer.reset();
        }
    }

    /// Sniper behavior - long range, precise shots
    fn sniper_behavior(
        &self,
        context: &BehaviorContext,
        ai: &mut AIBehavior,
        velocity: &mut Velocity,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        laser_sight: Option<&mut LaserSight>,
    ) {
        let config = ArchetypeConfig::for_archetype(*self);

        // Positioning behavior - maintain long distance
        if context.distance_to_player < config.preferred_distance {
            // Back away to maintain range
            velocity.linvel = -context.direction_to_player * config.speed;
        } else {
            // Stop moving when at good range
            velocity.linvel = Vec2::ZERO;
        }

        // Laser sight behavior
        if let Some(laser) = laser_sight {
            let in_range = context.distance_to_player <= config.preferred_distance;
            let ready_to_shoot = ai.timer.remaining().as_secs_f32() < 1.0; // Show laser 1 second before shooting

            laser.is_active = in_range && ready_to_shoot;
            if laser.is_active {
                // Target the player's position
                laser.target_pos = context.enemy_pos + context.direction_to_player * context.distance_to_player;
            }
        }

        // Shooting behavior
        if ai.timer.finished() && context.distance_to_player <= config.preferred_distance {
            let bullet_velocity = context.direction_to_player * SNIPER_BULLET_SPEED;
            spawn_enemy_bullet(commands, meshes, materials, context.enemy_pos, bullet_velocity, Color::srgb(0.0, 1.0, 0.5));
            ai.timer.reset();
        }
    }

    /// Machine gunner behavior - medium range, rapid fire
    fn machine_gunner_behavior(
        &self,
        context: &BehaviorContext,
        ai: &mut AIBehavior,
        velocity: &mut Velocity,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        let config = ArchetypeConfig::for_archetype(*self);

        // Positioning behavior
        if context.distance_to_player > config.preferred_distance + 30.0 {
            // Move closer
            velocity.linvel = context.direction_to_player * config.speed;
        } else if context.distance_to_player < config.preferred_distance - 30.0 {
            // Back away
            velocity.linvel = -context.direction_to_player * config.speed;
        } else {
            // At good distance, slow movement
            velocity.linvel = context.direction_to_player * config.speed * 0.2;
        }

        // Rapid fire behavior
        if ai.timer.finished() {
            // Add jitter/spread to machine gun bullets for realistic spray
            let jitter_angle = (fastrand::f32() - 0.5) * 0.2; // ±0.1 radians (~±6 degrees)
            let jittered_direction = Vec2::new(
                context.direction_to_player.x * jitter_angle.cos() - context.direction_to_player.y * jitter_angle.sin(),
                context.direction_to_player.x * jitter_angle.sin() + context.direction_to_player.y * jitter_angle.cos(),
            );
            let bullet_velocity = jittered_direction * ENEMY_BULLET_SPEED;
            spawn_enemy_bullet(commands, meshes, materials, context.enemy_pos, bullet_velocity, Color::srgb(0.8, 0.2, 0.8));
            ai.timer.reset();
        }
    }
}

/// Spawns enemies periodically around the map edges
pub fn spawn_enemies(
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

            // Choose random archetype and get its configuration
            let archetype = match fastrand::usize(0..5) {
                0 => EnemyArchetype::SmallMelee,
                1 => EnemyArchetype::BigMelee,
                2 => EnemyArchetype::Shotgunner,
                3 => EnemyArchetype::Sniper,
                _ => EnemyArchetype::MachineGunner,
            };

            let config = ArchetypeConfig::for_archetype(archetype);

            // Spawn enemy with archetype-specific properties
            let mut entity_commands = commands.spawn((
                Mesh2d(meshes.add(Circle::new(config.radius))),
                MeshMaterial2d(materials.add(config.color)),
                Transform::from_translation(spawn_pos.extend(0.0)),
                Enemy { archetype },
                Team::Enemy,
                Health::new(config.health),
                AIBehavior::new(config.preferred_distance, config.fire_rate),
                RigidBody::Dynamic,
                Collider::ball(config.radius),
                LockedAxes::ROTATION_LOCKED,
                Velocity::zero(),
                // Enable collision events for damage detection
                ActiveEvents::COLLISION_EVENTS,
            ));

            // Add laser sight component for snipers
            if matches!(archetype, EnemyArchetype::Sniper) {
                entity_commands.insert(LaserSight {
                    is_active: false,
                    target_pos: Vec2::ZERO,
                });
            }

            // Reset the spawn timer
            spawn_timer.timer.reset();
        }
    }
}

/// Gets a random spawn position for enemies around the map edges, away from the player
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

/// AI system that controls enemy behavior based on their archetype
pub fn enemy_ai(
    mut enemy_query: Query<(
        &Transform,
        &mut Velocity,
        &Enemy,
        &mut AIBehavior,
        Option<&mut LaserSight>
    ), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation.truncate();

        for (enemy_transform, mut enemy_velocity, enemy, mut ai_behavior, mut laser_sight) in enemy_query.iter_mut() {
            let enemy_pos = enemy_transform.translation.truncate();
            let distance_to_player = enemy_pos.distance(player_pos);
            let direction_to_player = (player_pos - enemy_pos).normalize_or_zero();

            // Create behavior context
            let context = BehaviorContext {
                enemy_pos,
                distance_to_player,
                direction_to_player,
            };

            // Update AI timer
            ai_behavior.timer.tick(time.delta());

            // Execute archetype-specific behavior
            enemy.archetype.execute_behavior(
                &context,
                &mut ai_behavior,
                &mut enemy_velocity,
                &mut commands,
                &mut meshes,
                &mut materials,
                laser_sight.as_deref_mut(),
            );
        }
    }
}

/// Spawns a spread of shotgun pellets
fn spawn_shotgun_spread(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    spawn_pos: Vec2,
    base_direction: Vec2,
) {
    for i in 0..SHOTGUNNER_PELLETS {
        let spread_angle = (i as f32 - (SHOTGUNNER_PELLETS as f32 - 1.0) / 2.0) * 0.15; // 0.15 radians spread
        let direction = Vec2::new(
            base_direction.x * spread_angle.cos() - base_direction.y * spread_angle.sin(),
            base_direction.x * spread_angle.sin() + base_direction.y * spread_angle.cos(),
        );

        let bullet_velocity = direction * SHOTGUN_BULLET_SPEED;
        spawn_enemy_bullet(commands, meshes, materials, spawn_pos, bullet_velocity, Color::srgb(1.0, 0.7, 0.0));
    }
}

/// Spawns a generic enemy bullet with specified properties
fn spawn_enemy_bullet(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    spawn_pos: Vec2,
    velocity: Vec2,
    color: Color,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PROJECTILE_SIZE * 0.8))), // Slightly smaller than player bullets
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(spawn_pos.extend(0.1)),
        Projectile {
            lifetime: Timer::from_seconds(ENEMY_BULLET_LIFETIME, TimerMode::Once),
            team: Team::Enemy,
        },
        RigidBody::Dynamic,
        Collider::ball(PROJECTILE_SIZE * 0.8),
        Velocity::linear(velocity),
        ActiveEvents::COLLISION_EVENTS,
    ));
}

/// System to render laser sights for snipers
pub fn laser_sight_system(
    mut gizmos: Gizmos,
    laser_query: Query<(&Transform, &LaserSight), With<Enemy>>,
) {
    for (transform, laser) in laser_query.iter() {
        if laser.is_active {
            let start_pos = transform.translation.truncate();
            let end_pos = laser.target_pos;

            // Draw red laser line
            gizmos.line_2d(start_pos, end_pos, Color::srgb(1.0, 0.0, 0.0));

            // Draw small targeting dot at the end
            gizmos.circle_2d(end_pos, 3.0, Color::srgb(1.0, 0.2, 0.0));
        }
    }
}
