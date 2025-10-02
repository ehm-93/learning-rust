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

/// Event fired when a grenade explodes
#[derive(Event)]
pub struct GrenadeExplosionEvent {
    pub position: Vec2,
    pub damage: f32,
    pub radius: f32,
    pub team: crate::components::Team,
}

/// Event fired when a portal is activated to transition to a scene
#[derive(Event)]
pub struct PortalActivationEvent {
    pub portal_id: crate::world::scenes::cathedral::PortalId,
    pub depth: u32,
    pub modifiers: Vec<String>,
}
