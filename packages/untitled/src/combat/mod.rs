//! New combat system - generic effects foundation
//!
//! This module replaces the old hardcoded projectile/grenade system with a
//! data-driven effect system that can handle any type of combat interaction.

pub mod effects;
pub mod resolver;

pub use effects::*;
pub use resolver::*;
