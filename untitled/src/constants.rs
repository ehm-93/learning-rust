// Player constants
pub const PLAYER_SPEED: f32 = 200.0;
pub const PLAYER_RADIUS: f32 = 10.0;
pub const PLAYER_MAX_HEALTH: f32 = 100.0;

// Dash constants
pub const DASH_SPEED: f32 = 800.0;
pub const DASH_DURATION: f32 = 0.2; // 0.2 seconds
pub const DASH_COOLDOWN: f32 = 1.5; // 1.5 seconds cooldown
pub const DASH_IFRAME_DURATION: f32 = 0.15; // Invincibility for most of dash

// Projectile constants
pub const PROJECTILE_SPEED: f32 = 800.0;
pub const PROJECTILE_SIZE: f32 = 3.0;
pub const PROJECTILE_LIFETIME: f32 = 3.0;
pub const PROJECTILE_DAMAGE: f32 = 10.0;
pub const FIRE_RATE: f32 = 0.1; // 10 shots per second

// World constants
pub const MAP_WIDTH: f32 = 1200.0;
pub const MAP_HEIGHT: f32 = 900.0;
pub const WALL_THICKNESS: f32 = 20.0;

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
pub const SNIPER_RANGE: f32 = 500.0;
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

// Hit flash constants
pub const HIT_FLASH_DURATION: f32 = 0.15; // Duration of hit flash in seconds

// Grenade constants
pub const GRENADE_SPEED: f32 = 400.0;
pub const GRENADE_SIZE: f32 = 4.0;
pub const GRENADE_FUSE_TIME: f32 = 1.5; // Time before explosion
pub const GRENADE_DAMAGE: f32 = 100.0;
pub const GRENADE_EXPLOSION_RADIUS: f32 = 120.0;
pub const GRENADE_THROW_COOLDOWN: f32 = 3.0; // 3 seconds between grenades
pub const GRENADE_BOUNCE: f32 = 0.7; // Restitution coefficient (bounciness)
pub const GRENADE_DAMPING: f32 = 0.75; // Damping coefficient
pub const GRENADE_MIN_SPEED: f32 = 75.0; // Minimum speed before grenade stops moving

// Explosion visual constants
pub const EXPLOSION_DURATION: f32 = 0.3; // How long explosion animation lasts
pub const EXPLOSION_START_SIZE: f32 = 5.0; // Initial explosion radius
pub const EXPLOSION_END_SIZE: f32 = 120.0; // Final explosion radius

// Grid-based dungeon generation constants
pub const GRID_WIDTH: usize = 64; // M: Grid width in cells
pub const GRID_HEIGHT: usize = 64; // N: Grid height in cells
pub const CELL_SIZE: f32 = 120.0; // Physical size of each grid cell
pub const ROOM_COUNT: usize = 8; // Number of rooms to generate
pub const ROOM_SIZE_X: usize = 6; // I: Room width in grid cells
pub const ROOM_SIZE_Y: usize = 6; // J: Room height in grid cells

// Dungeon visual constants
pub const ROOM_FLOOR_COLOR: [f32; 3] = [0.4, 0.3, 0.25]; // Darker, warmer brown for rooms
pub const CORRIDOR_FLOOR_COLOR: [f32; 3] = [0.3, 0.35, 0.4]; // Cooler bluish-gray for corridors
pub const WALL_COLOR: [f32; 3] = [0.6, 0.6, 0.7]; // Existing wall color

// Line of sight constants
pub const LOS_MAX_RANGE: f32 = 800.0; // Maximum line of sight range
