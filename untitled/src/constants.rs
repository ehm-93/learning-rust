// Player constants
pub const PLAYER_SPEED: f32 = 200.0;
pub const PLAYER_RADIUS: f32 = 10.0;
pub const PLAYER_MAX_HEALTH: f32 = 100.0;

// Projectile constants
pub const PROJECTILE_SPEED: f32 = 800.0;
pub const PROJECTILE_SIZE: f32 = 3.0;
pub const PROJECTILE_LIFETIME: f32 = 3.0;
pub const PROJECTILE_DAMAGE: f32 = 10.0;
pub const FIRE_RATE: f32 = 0.2; // 5 shots per second

// World constants
pub const MAP_WIDTH: f32 = 1200.0;
pub const MAP_HEIGHT: f32 = 900.0;
pub const WALL_THICKNESS: f32 = 20.0;
pub const OBSTACLE_WIDTH: f32 = 40.0;
pub const OBSTACLE_HEIGHT: f32 = 80.0;

// Camera constants
pub const CAMERA_FOLLOW_SPEED: f32 = 5.0;
pub const CURSOR_BIAS_STRENGTH: f32 = 1.0;
pub const CURSOR_BIAS_DISTANCE: f32 = 150.0;

// Combat constants
pub const KNOCKBACK_FORCE: f32 = 200.0;
pub const ENEMY_CONTACT_DAMAGE: f32 = 25.0;

// Enemy archetype constants
pub const SMALL_MELEE_HEALTH: f32 = 10.0;
pub const SMALL_MELEE_SPEED: f32 = 250.0;
pub const SMALL_MELEE_RADIUS: f32 = 6.0;

pub const BIG_MELEE_HEALTH: f32 = 80.0;
pub const BIG_MELEE_SPEED: f32 = 60.0;
pub const BIG_MELEE_RADIUS: f32 = 18.0;

pub const SHOTGUNNER_HEALTH: f32 = 30.0;
pub const SHOTGUNNER_SPEED: f32 = 140.0;
pub const SHOTGUNNER_RADIUS: f32 = 10.0;
pub const SHOTGUNNER_RANGE: f32 = 80.0;
pub const SHOTGUNNER_FIRE_RATE: f32 = 2.0;
pub const SHOTGUNNER_PELLETS: usize = 5;

pub const SNIPER_HEALTH: f32 = 20.0;
pub const SNIPER_SPEED: f32 = 100.0;
pub const SNIPER_RADIUS: f32 = 8.0;
pub const SNIPER_RANGE: f32 = 300.0;
pub const SNIPER_FIRE_RATE: f32 = 1.5;

pub const MACHINE_GUNNER_HEALTH: f32 = 40.0;
pub const MACHINE_GUNNER_SPEED: f32 = 160.0;
pub const MACHINE_GUNNER_RADIUS: f32 = 9.0;
pub const MACHINE_GUNNER_RANGE: f32 = 150.0;
pub const MACHINE_GUNNER_FIRE_RATE: f32 = 0.15;

// Enemy spawn constants
pub const ENEMY_SPAWN_RATE: f32 = 2.0;
pub const MAX_ENEMIES: usize = 8;

// Enemy projectile constants
pub const ENEMY_BULLET_SPEED: f32 = 600.0;
pub const ENEMY_BULLET_DAMAGE: f32 = 15.0;
pub const ENEMY_BULLET_LIFETIME: f32 = 2.0;
pub const SNIPER_BULLET_SPEED: f32 = 1200.0;
pub const SHOTGUN_BULLET_SPEED: f32 = 400.0;
