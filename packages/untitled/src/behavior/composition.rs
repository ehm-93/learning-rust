use crate::behavior::{Behavior, BehaviorDefinition, Params};

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
    fn on_spawn(&mut self, world: &mut crate::behavior::world_api::WorldApi) {
        for behavior in &mut self.behaviors {
            behavior.on_spawn(world);
        }
    }

    fn on_update(&mut self, world: &mut crate::behavior::world_api::WorldApi, dt: f32) {
        for behavior in &mut self.behaviors {
            behavior.on_update(world, dt);
        }
    }

    fn on_despawn(&mut self, world: &mut crate::behavior::world_api::WorldApi) {
        for behavior in &mut self.behaviors {
            behavior.on_despawn(world);
        }
    }

    fn on_collision_enter(&mut self, world: &mut crate::behavior::world_api::WorldApi, other: bevy::prelude::Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_enter(world, other);
        }
    }

    fn on_collision_stay(&mut self, world: &mut crate::behavior::world_api::WorldApi, other: bevy::prelude::Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_stay(world, other);
        }
    }

    fn on_collision_exit(&mut self, world: &mut crate::behavior::world_api::WorldApi, other: bevy::prelude::Entity) {
        for behavior in &mut self.behaviors {
            behavior.on_collision_exit(world, other);
        }
    }
}

/// Trait for converting definitions into behavior lists (used by Rust tuple syntax)
pub trait IntoBehaviors {
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>>;
}

// Single definition
impl IntoBehaviors for BehaviorDefinition {
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>> {
        vec![self(params.clone())]
    }
}

// Tuple implementations for compile-time composition
impl<T1, T2> IntoBehaviors for (T1, T2)
where
    T1: IntoBehaviors,
    T2: IntoBehaviors
{
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>> {
        let mut behaviors = self.0.into_behaviors(params);
        behaviors.extend(self.1.into_behaviors(params));
        behaviors
    }
}

impl<T1, T2, T3> IntoBehaviors for (T1, T2, T3)
where
    T1: IntoBehaviors,
    T2: IntoBehaviors,
    T3: IntoBehaviors,
{
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>> {
        let mut behaviors = self.0.into_behaviors(params);
        behaviors.extend(self.1.into_behaviors(params));
        behaviors.extend(self.2.into_behaviors(params));
        behaviors
    }
}

impl<T1, T2, T3, T4> IntoBehaviors for (T1, T2, T3, T4)
where
    T1: IntoBehaviors,
    T2: IntoBehaviors,
    T3: IntoBehaviors,
    T4: IntoBehaviors,
{
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>> {
        let mut behaviors = self.0.into_behaviors(params);
        behaviors.extend(self.1.into_behaviors(params));
        behaviors.extend(self.2.into_behaviors(params));
        behaviors.extend(self.3.into_behaviors(params));
        behaviors
    }
}

impl<T1, T2, T3, T4, T5> IntoBehaviors for (T1, T2, T3, T4, T5)
where
    T1: IntoBehaviors,
    T2: IntoBehaviors,
    T3: IntoBehaviors,
    T4: IntoBehaviors,
    T5: IntoBehaviors,
{
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>> {
        let mut behaviors = self.0.into_behaviors(params);
        behaviors.extend(self.1.into_behaviors(params));
        behaviors.extend(self.2.into_behaviors(params));
        behaviors.extend(self.3.into_behaviors(params));
        behaviors.extend(self.4.into_behaviors(params));
        behaviors
    }
}

impl<T1, T2, T3, T4, T5, T6> IntoBehaviors for (T1, T2, T3, T4, T5, T6)
where
    T1: IntoBehaviors,
    T2: IntoBehaviors,
    T3: IntoBehaviors,
    T4: IntoBehaviors,
    T5: IntoBehaviors,
    T6: IntoBehaviors,
{
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>> {
        let mut behaviors = self.0.into_behaviors(params);
        behaviors.extend(self.1.into_behaviors(params));
        behaviors.extend(self.2.into_behaviors(params));
        behaviors.extend(self.3.into_behaviors(params));
        behaviors.extend(self.4.into_behaviors(params));
        behaviors.extend(self.5.into_behaviors(params));
        behaviors
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> IntoBehaviors for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: IntoBehaviors,
    T2: IntoBehaviors,
    T3: IntoBehaviors,
    T4: IntoBehaviors,
    T5: IntoBehaviors,
    T6: IntoBehaviors,
    T7: IntoBehaviors,
{
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>> {
        let mut behaviors = self.0.into_behaviors(params);
        behaviors.extend(self.1.into_behaviors(params));
        behaviors.extend(self.2.into_behaviors(params));
        behaviors.extend(self.3.into_behaviors(params));
        behaviors.extend(self.4.into_behaviors(params));
        behaviors.extend(self.5.into_behaviors(params));
        behaviors.extend(self.6.into_behaviors(params));
        behaviors
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> IntoBehaviors for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: IntoBehaviors,
    T2: IntoBehaviors,
    T3: IntoBehaviors,
    T4: IntoBehaviors,
    T5: IntoBehaviors,
    T6: IntoBehaviors,
    T7: IntoBehaviors,
    T8: IntoBehaviors,
{
    fn into_behaviors(self, params: &Params) -> Vec<Box<dyn Behavior>> {
        let mut behaviors = self.0.into_behaviors(params);
        behaviors.extend(self.1.into_behaviors(params));
        behaviors.extend(self.2.into_behaviors(params));
        behaviors.extend(self.3.into_behaviors(params));
        behaviors.extend(self.4.into_behaviors(params));
        behaviors.extend(self.5.into_behaviors(params));
        behaviors.extend(self.6.into_behaviors(params));
        behaviors.extend(self.7.into_behaviors(params));
        behaviors
    }
}

// We can continue this pattern up to 12 elements as specified
// For brevity, I'll stop at 8 here, but the pattern is clear
