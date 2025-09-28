use bevy::prelude::*;

/// Event fired when a projectile impacts something
#[derive(Event)]
pub struct ProjectileImpactEvent {
    pub projectile: Entity,
    pub target: Entity,
}

/// Event fired when an entity should take damage
#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub damage: f32,
}

/// Event fired when an enemy should flash on hit
#[derive(Event)]
pub struct HitFlashEvent {
    pub target: Entity,
}
