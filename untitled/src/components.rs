use bevy::prelude::*;

/// Team affiliation for entities - determines collision and damage interactions
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Player,
    Enemy,
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
}

impl AIBehavior {
    pub fn new(behavior_interval: f32) -> Self {
        Self {
            timer: Timer::from_seconds(behavior_interval, TimerMode::Repeating),
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

/// Main camera marker component
#[derive(Component)]
pub struct MainCamera;

/// Laser sight component for snipers
#[derive(Component)]
pub struct LaserSight {
    pub is_active: bool,
    pub target_pos: Vec2,
}

/// Health bar UI component
#[derive(Component)]
pub struct HealthBar;

/// Dash ability component
#[derive(Component)]
pub struct Dash {
    pub cooldown_timer: Timer,
    pub dash_timer: Timer,
    pub iframe_timer: Timer,
    pub is_dashing: bool,
    pub is_invincible: bool,
    pub dash_direction: Vec2,
}

impl Dash {
    pub fn new() -> Self {
        use crate::constants::*;
        Self {
            cooldown_timer: Timer::from_seconds(DASH_COOLDOWN, TimerMode::Once),
            dash_timer: Timer::from_seconds(DASH_DURATION, TimerMode::Once),
            iframe_timer: Timer::from_seconds(DASH_IFRAME_DURATION, TimerMode::Once),
            is_dashing: false,
            is_invincible: false,
            dash_direction: Vec2::ZERO,
        }
    }

    pub fn can_dash(&self) -> bool {
        self.cooldown_timer.finished() && !self.is_dashing
    }

    pub fn start_dash(&mut self, direction: Vec2) {
        if self.can_dash() {
            self.is_dashing = true;
            self.is_invincible = true;
            self.dash_direction = direction.normalize_or_zero();
            self.dash_timer.reset();
            self.iframe_timer.reset();
            self.cooldown_timer.reset();
        }
    }
}

/// Score display UI component
#[derive(Component)]
pub struct ScoreText;

/// Game over overlay UI component
#[derive(Component)]
pub struct GameOverOverlay;

/// Restart button UI component
#[derive(Component)]
pub struct RestartButton;

/// Hit flash component for visual damage feedback
#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
    pub original_color: Color,
}

/// Grenade component with fuse timer
#[derive(Component)]
pub struct Grenade {
    pub fuse_timer: Timer,
    pub team: Team,
}

/// Grenade throwing capability component
#[derive(Component)]
pub struct GrenadeThrower {
    pub cooldown_timer: Timer,
}

impl GrenadeThrower {
    pub fn new() -> Self {
        use crate::constants::*;
        Self {
            cooldown_timer: Timer::from_seconds(GRENADE_THROW_COOLDOWN, TimerMode::Once),
        }
    }

    pub fn can_throw(&self) -> bool {
        self.cooldown_timer.finished()
    }

    pub fn throw_grenade(&mut self) {
        self.cooldown_timer.reset();
    }
}

/// Explosion visual effect component
#[derive(Component)]
pub struct ExplosionEffect {
    pub timer: Timer,
    pub start_radius: f32,
    pub end_radius: f32,
}

/// Dungeon wall marker component
#[derive(Component)]
pub struct DungeonWall;

/// Line of sight component for tracking visibility to targets
#[derive(Component)]
pub struct LineOfSight {
    pub has_los_to_player: bool,
    pub last_known_player_position: Option<Vec2>,
    pub los_check_timer: Timer,
}

impl LineOfSight {
    pub fn new() -> Self {
        Self {
            has_los_to_player: false,
            last_known_player_position: None,
            los_check_timer: Timer::from_seconds(0.1, TimerMode::Repeating), // Check LOS 10 times per second
        }
    }
}
