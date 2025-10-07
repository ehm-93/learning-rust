//! Built-in foundational behaviors
//!
//! These are core, well-tested behaviors implemented in Rust that provide
//! fundamental functionality for entities.
//!
//! ## Available Behaviors
//!
//! - **lifetime**: Despawns the entity after a specified duration
//! - **constant_velocity**: Maintains a constant velocity vector
//! - **spin**: Applies constant angular velocity (rotation)
//! - **acceleration**: Applies acceleration in a direction with optional max speed
//!
//! ## Usage
//!
//! These behaviors are automatically registered and can be used from both Rust
//! and Lua code. See individual behavior documentation for parameter details.

mod lifetime;
mod constant_velocity;
mod spin;
mod acceleration;

pub use lifetime::LifetimeBehavior;
pub use constant_velocity::ConstantVelocityBehavior;
pub use spin::SpinBehavior;
pub use acceleration::AccelerationBehavior;

use crate::behavior::BehaviorRegistry;

/// Register all builtin behaviors with the registry
pub fn register_builtins(registry: &mut BehaviorRegistry) {
    registry.register("lifetime", Box::new(|params| {
        Box::new(LifetimeBehavior::new(params))
    }));

    registry.register("constant_velocity", Box::new(|params| {
        Box::new(ConstantVelocityBehavior::new(params))
    }));

    registry.register("spin", Box::new(|params| {
        Box::new(SpinBehavior::new(params))
    }));

    registry.register("acceleration", Box::new(|params| {
        Box::new(AccelerationBehavior::new(params))
    }));
}
