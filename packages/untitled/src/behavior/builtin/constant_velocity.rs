use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::behavior::{Behavior, Params};

/// Maintain constant velocity
#[derive(Debug, Clone)]
pub struct ConstantVelocityBehavior {
    velocity: Vec2,
    entity: Option<Entity>,
}

impl ConstantVelocityBehavior {
    /// Create a new ConstantVelocityBehavior
    ///
    /// # Parameters
    /// - `vx` (f32): X component of velocity in units/second (default: 0.0)
    /// - `vy` (f32): Y component of velocity in units/second (default: 0.0)
    pub fn new(params: Params) -> Self {
        let vx = params.get_f32("vx").unwrap_or(0.0);
        let vy = params.get_f32("vy").unwrap_or(0.0);

        Self {
            velocity: Vec2::new(vx, vy),
            entity: None,
        }
    }
}

impl Behavior for ConstantVelocityBehavior {
    fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    fn on_spawn(&mut self, world: &mut World) {
        info!("[ConstantVelocity] Setting velocity to ({:.1}, {:.1})",
            self.velocity.x, self.velocity.y);

        if let Some(entity) = self.entity {
            if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
                velocity.linvel = self.velocity;
            }
        }
    }

    fn on_update(&mut self, world: &mut World, _dt: f32) {
        if let Some(entity) = self.entity {
            if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
                velocity.linvel = self.velocity;
            }
        }
    }
}
