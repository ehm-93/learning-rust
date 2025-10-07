use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::behavior::{Behavior, Params};

/// Apply acceleration in a direction with optional max speed
#[derive(Debug, Clone)]
pub struct AccelerationBehavior {
    direction: Vec2,
    acceleration: f32,
    max_speed: Option<f32>,
    entity: Option<Entity>,
}

impl AccelerationBehavior {
    /// Create a new AccelerationBehavior
    ///
    /// # Parameters
    /// - `dx` (f32): X component of acceleration direction (default: 1.0)
    /// - `dy` (f32): Y component of acceleration direction (default: 0.0)
    /// - `acceleration` (f32): Acceleration magnitude in units/second² (default: 100.0)
    /// - `max_speed` (f32, optional): Maximum speed cap in units/second (default: none)
    ///
    /// Note: The direction vector (dx, dy) will be normalized automatically
    pub fn new(params: Params) -> Self {
        let dx = params.get_f32("dx").unwrap_or(1.0);
        let dy = params.get_f32("dy").unwrap_or(0.0);
        let direction = Vec2::new(dx, dy).normalize_or_zero();

        let acceleration = params.get_f32("acceleration").unwrap_or(100.0);
        let max_speed = params.get_f32("max_speed").ok();        Self {
            direction,
            acceleration,
            max_speed,
            entity: None,
        }
    }
}

impl Behavior for AccelerationBehavior {
    fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    fn on_spawn(&mut self, _world: &mut World) {
        info!("[Acceleration] dir: ({:.2}, {:.2}), accel: {:.1}, max_speed: {:?}",
            self.direction.x, self.direction.y, self.acceleration, self.max_speed);
    }

    fn on_update(&mut self, world: &mut World, dt: f32) {
        if let Some(entity) = self.entity {
            if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
                // Apply acceleration
                velocity.linvel += self.direction * self.acceleration * dt;

                // Apply max speed if set
                if let Some(max_speed) = self.max_speed {
                    let speed = velocity.linvel.length();
                    if speed > max_speed {
                        velocity.linvel = velocity.linvel.normalize() * max_speed;
                    }
                }
            }
        }
    }
}
