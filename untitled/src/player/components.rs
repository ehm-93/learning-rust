use bevy::prelude::*;

/// Player marker component
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Dash ability component
#[derive(Component, Reflect)]
#[reflect(Component)]
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

/// Grenade throwing capability component
#[derive(Component, Reflect)]
#[reflect(Component)]
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

/// Bundle of all core player components for easy spawning
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub team: crate::components::Team,
    pub health: crate::components::Health,
    pub dash: Dash,
    pub grenade_thrower: GrenadeThrower,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        use crate::constants::*;
        Self {
            player: Player,
            team: crate::components::Team::Player,
            health: crate::components::Health::new(PLAYER_MAX_HEALTH),
            dash: Dash::new(),
            grenade_thrower: GrenadeThrower::new(),
        }
    }
}
