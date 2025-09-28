use bevy::prelude::*;

/// Team affiliation for entities - determines collision and damage interactions
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Player,
    Enemy,
    Neutral,
}

/// Player marker component
#[derive(Component)]
pub struct Player;

/// Enemy archetype defining behavior and stats
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyArchetype {
    SmallMelee,
    BigMelee,
    Shotgunner,
    Sniper,
    MachineGunner,
}

/// Enemy marker component with archetype
#[derive(Component)]
pub struct Enemy {
    pub archetype: EnemyArchetype,
}

/// AI behavior state for enemies
#[derive(Component)]
pub struct AIBehavior {
    pub timer: Timer,
    pub preferred_distance: f32,
}

impl AIBehavior {
    pub fn new(preferred_distance: f32, behavior_interval: f32) -> Self {
        Self {
            timer: Timer::from_seconds(behavior_interval, TimerMode::Repeating),
            preferred_distance,
        }
    }
}

/// Projectile component with lifetime and team affiliation
#[derive(Component)]
pub struct Projectile {
    pub lifetime: Timer,
    pub team: Team,
}

/// Health component for entities that can take damage
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max_health: f32) -> Self {
        Self {
            current: max_health,
            max: max_health,
        }
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

/// Static obstacle marker component
#[derive(Component)]
pub struct Obstacle;

/// Main camera marker component
#[derive(Component)]
pub struct MainCamera;

/// Map boundary marker component
#[derive(Component)]
pub struct Boundary;

/// Laser sight component for snipers
#[derive(Component)]
pub struct LaserSight {
    pub is_active: bool,
    pub target_pos: Vec2,
}

/// Health bar UI component
#[derive(Component)]
pub struct HealthBar;
