use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::behavior::{Behavior, Params};

/// Constant angular velocity (spinning)
#[derive(Debug, Clone)]
pub struct SpinBehavior {
    angular_velocity: f32,
    entity: Option<Entity>,
}

impl SpinBehavior {
    /// Create a new SpinBehavior
    ///
    /// # Parameters
    /// - `angular_velocity` or `speed` (f32): Rotation speed in radians/second (default: 1.0)
    ///   - Positive values rotate counter-clockwise
    ///   - Negative values rotate clockwise
    pub fn new(params: Params) -> Self {
        let angular_velocity = params.get_f32("angular_velocity")
            .or_else(|_| params.get_f32("speed"))
            .unwrap_or(1.0);

        Self {
            angular_velocity,
            entity: None,
        }
    }
}

impl Behavior for SpinBehavior {
    fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    fn on_spawn(&mut self, _world: &mut World) {
        info!("[Spin] Angular velocity: {:.2} rad/s", self.angular_velocity);
    }

    fn on_update(&mut self, world: &mut World, _dt: f32) {
        if let Some(entity) = self.entity {
            if let Some(mut velocity) = world.get_mut::<Velocity>(entity) {
                velocity.angvel = self.angular_velocity;
            }
        }
    }
}
