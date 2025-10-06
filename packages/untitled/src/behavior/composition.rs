use bevy::prelude::*;

use crate::behavior::Behavior;

/// CompositeBehavior composes multiple behaviors at runtime (used by Lua)
pub struct CompositeBehavior {
    behaviors: Vec<Box<dyn Behavior>>,
}

impl CompositeBehavior {
    /// Create a new CompositeBehavior with the given behaviors
    pub fn new(behaviors: Vec<Box<dyn Behavior>>) -> Self {
        Self { behaviors }
    }
}

impl Behavior for CompositeBehavior {
    fn on_spawn(&mut self, world: &mut World) {
        for behavior in &mut self.behaviors {
            behavior.on_spawn(world);
        }
    }

    fn on_update(&mut self, world: &mut World, dt: f32) {
        for behavior in &mut self.behaviors {
            behavior.on_update(world, dt);
        }
    }

    fn on_despawn(&mut self, world: &mut World) {
        for behavior in &mut self.behaviors {
            behavior.on_despawn(world);
        }
    }

    fn on_collision_enter(&mut self, world: &mut World, other: Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_enter(world, other);
        }
    }

    fn on_collision_stay(&mut self, world: &mut World, other: Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_stay(world, other);
        }
    }

    fn on_collision_exit(&mut self, world: &mut World, other: Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_exit(world, other);
        }
    }
}
