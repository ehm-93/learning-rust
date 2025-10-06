use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::behavior::{Behavior, Params, world_api::WorldApi};
use crate::components::{Projectile, Team};

/// Movement behavior - uses on_spawn to add velocity
pub struct MovementBehavior {
    speed: f32,
    direction: Vec3,
}

impl Behavior for MovementBehavior {
    fn on_spawn(&mut self, world: &mut WorldApi) {
        let velocity = self.direction.normalize() * self.speed;
        world.add_component(Velocity {
            linvel: Vec2::new(velocity.x, velocity.y),
            angvel: 0.0,
        });
    }
}

/// Create a MovementBehavior from parameters
pub fn movement_def(params: Params) -> Box<dyn Behavior> {
    Box::new(MovementBehavior {
        speed: params.get_f32("speed").unwrap_or(10.0),
        direction: params.get_vec3("direction").unwrap_or(Vec3::X),
    })
}

/// Homing behavior - uses on_update to adjust velocity toward target
pub struct HomingBehavior {
    target: Entity,
    strength: f32,
}

impl Behavior for HomingBehavior {
    fn on_update(&mut self, world: &mut WorldApi, dt: f32) {
        if let Some(target_transform) = world.get_entity_transform(self.target) {
            if let Some(current_transform) = world.get_transform() {
                let direction = (target_transform.translation - current_transform.translation).normalize();

                // Modify velocity to home toward target
                world.modify_component::<Velocity>(|velocity| {
                    let current_speed = velocity.linvel.length();
                    let target_velocity = Vec2::new(direction.x, direction.y) * current_speed;
                    velocity.linvel = velocity.linvel.lerp(target_velocity, self.strength * dt);
                });
            }
        }
    }
}

/// Create a HomingBehavior from parameters
pub fn homing_def(params: Params) -> Box<dyn Behavior> {
    Box::new(HomingBehavior {
        target: params.get_entity("target").unwrap_or(Entity::PLACEHOLDER),
        strength: params.get_f32("strength").unwrap_or(5.0),
    })
}

/// OnHit behavior - uses collision hooks to spawn behaviors and despawn
pub struct OnHitBehavior {
    spawn_behavior: String,
    damage: f32,
}

impl Behavior for OnHitBehavior {
    fn on_collision_enter(&mut self, world: &mut WorldApi, _other: Entity) {
        // Spawn the specified behavior at current position
        if let Some(transform) = world.get_transform() {
            let spawn_params = Params::from([
                ("position", transform.translation.into()),
                ("damage", self.damage.into()),
            ]);
            world.spawn_behavior(&self.spawn_behavior, spawn_params);
        }

        // Despawn this entity
        world.despawn();
    }
}

/// Create an OnHitBehavior from parameters
pub fn on_hit_def(params: Params) -> Box<dyn Behavior> {
    Box::new(OnHitBehavior {
        spawn_behavior: params.get_behavior_id("spawn_behavior").unwrap_or("explosion".to_string()),
        damage: params.get_f32("damage").unwrap_or(10.0),
    })
}

/// DamageArea behavior - damages entities in radius using on_spawn
pub struct DamageAreaBehavior {
    radius: f32,
    damage: f32,
    team: Team,
}

impl Behavior for DamageAreaBehavior {
    fn on_spawn(&mut self, world: &mut WorldApi) {
        // Add a circular collider for the damage area
        world.add_component(Collider::ball(self.radius));
        world.add_component(Sensor);
        world.add_component(self.team);

        // We'll handle the actual damage dealing in collision events
        // For now, just mark this as a damage area
        world.add_component(DamageArea {
            damage: self.damage,
            radius: self.radius,
        });
    }
}

/// Marker component for damage areas
#[derive(Component)]
pub struct DamageArea {
    pub damage: f32,
    pub radius: f32,
}

/// Create a DamageAreaBehavior from parameters
pub fn damage_area_def(params: Params) -> Box<dyn Behavior> {
    Box::new(DamageAreaBehavior {
        radius: params.get_f32("radius").unwrap_or(5.0),
        damage: params.get_f32("damage").unwrap_or(100.0),
        team: Team::Player, // Default team
    })
}

/// Timeout behavior - uses on_update to track duration and despawn
pub struct TimeoutBehavior {
    duration: f32,
    timer: Timer,
}

impl TimeoutBehavior {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

impl Behavior for TimeoutBehavior {
    fn on_update(&mut self, world: &mut WorldApi, dt: f32) {
        self.timer.tick(std::time::Duration::from_secs_f32(dt));

        if self.timer.finished() {
            world.despawn();
        }
    }
}

/// Create a TimeoutBehavior from parameters
pub fn timeout_def(params: Params) -> Box<dyn Behavior> {
    let duration = params.get_f32("duration").unwrap_or(1.0);
    Box::new(TimeoutBehavior::new(duration))
}

/// Projectile behavior - adds projectile component and handles lifetime
pub struct ProjectileBehavior {
    lifetime: f32,
    team: Team,
}

impl Behavior for ProjectileBehavior {
    fn on_spawn(&mut self, world: &mut WorldApi) {
        world.add_component(Projectile {
            lifetime: Timer::from_seconds(self.lifetime, TimerMode::Once),
            team: self.team,
        });
    }

    fn on_update(&mut self, world: &mut WorldApi, dt: f32) {
        world.modify_component::<Projectile>(|projectile| {
            projectile.lifetime.tick(std::time::Duration::from_secs_f32(dt));
        });

        // Check if projectile has expired
        if let Some(projectile) = world.get_component::<Projectile>() {
            if projectile.lifetime.finished() {
                world.despawn();
            }
        }
    }
}

/// Create a ProjectileBehavior from parameters
pub fn projectile_def(params: Params) -> Box<dyn Behavior> {
    Box::new(ProjectileBehavior {
        lifetime: params.get_f32("lifetime").unwrap_or(5.0),
        team: Team::Player, // Default team, should be set via params
    })
}
