use bevy::prelude::*;
use crate::behavior::{Behavior, Params};

/// Despawn entity after a specified duration
#[derive(Debug, Clone)]
pub struct LifetimeBehavior {
    lifetime: f32,
    elapsed: f32,
    entity: Option<Entity>,
}

impl LifetimeBehavior {
    /// Create a new LifetimeBehavior
    ///
    /// # Parameters
    /// - `lifetime` (f32): Duration in seconds before despawning (default: 5.0)
    pub fn new(params: Params) -> Self {
        let lifetime = params.get_f32("lifetime").unwrap_or(5.0);

        Self {
            lifetime,
            elapsed: 0.0,
            entity: None,
        }
    }
}

impl Behavior for LifetimeBehavior {
    fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    fn on_spawn(&mut self, _world: &mut World) {
        info!("[LifetimeBehavior] Spawned with lifetime: {:.2}s", self.lifetime);
    }

    fn on_update(&mut self, world: &mut World, dt: f32) {
        self.elapsed += dt;

        if self.elapsed >= self.lifetime {
            info!("[LifetimeBehavior] Lifetime expired, despawning");
            if let Some(entity) = self.entity {
                if let Some(entity_ref) = world.get_entity_mut(entity).ok() {
                    entity_ref.despawn();
                }
            }
        }
    }
}
