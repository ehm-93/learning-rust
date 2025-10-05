use bevy::prelude::*;
use std::collections::HashMap;

/// Unique identifier for effect definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectDefId(pub u32);

/// Identifier for damage types - fully data-driven
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DamageType(pub u32);

/// Standard damage type IDs (these are just conveniences, not hardcoded behavior)
impl DamageType {
    pub const TRUE: DamageType = DamageType(0);
    pub const PHYSICAL: DamageType = DamageType(1);
    pub const MAGICAL: DamageType = DamageType(2);
}

/// Faction identifier for combat targeting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FactionId(pub u32);

/// Standard faction IDs
impl FactionId {
    pub const PLAYER: FactionId = FactionId(0);
    pub const ENEMY: FactionId = FactionId(1);
    pub const NEUTRAL: FactionId = FactionId(2);
}

/// Component that defines an entity's faction for combat targeting
#[derive(Component, Debug, Clone, Copy)]
pub struct Faction {
    pub id: FactionId,
}

/// Core event that drives all combat resolution
/// This replaces ProjectileImpactEvent, GrenadeExplosionEvent, etc.
#[derive(Event, Debug, Clone)]
pub struct EffectEvent {
    /// Entity that caused this effect (for damage multipliers, etc)
    pub source: Entity,
    /// Entities that will be affected by this effect
    pub targets: Vec<Entity>,
    /// Which effect definition to apply
    pub effect_id: EffectDefId,
    /// World positions where effects hit (for visuals)
    pub hit_positions: Vec<Vec2>,
}

/// Event emitted after damage calculation but before application
#[derive(Event, Debug, Clone)]
pub struct DamageEvent {
    pub target: Entity,
    pub damage: f32,
    pub damage_type: DamageType,
    pub source: Entity,
}

/// Event emitted after status effect calculation but before application
#[derive(Event, Debug, Clone)]
pub struct StatusEvent {
    pub target: Entity,
    pub status_id: StatusId,
    pub intensity: f32,
    pub duration: f32,
    pub source: Entity,
}

/// Identifier for different status effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusId(pub u32);

/// Standard status effect IDs (for future use)
impl StatusId {
    pub const SLOW: StatusId = StatusId(0);
    pub const STUN: StatusId = StatusId(1);
    pub const DAMAGE_OVER_TIME: StatusId = StatusId(2);
    pub const KNOCKBACK: StatusId = StatusId(3);
}

/// How status effects stack when applied multiple times
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackBehavior {
    /// Replace existing effect with new one
    Replace,
    /// Add to existing intensity/duration
    Stack,
    /// Keep highest intensity, refresh duration
    RefreshHighest,
}

/// Definition of what an effect does when triggered
/// This will be expanded in later phases, but starts simple
#[derive(Debug, Clone)]
pub struct EffectDefinition {
    pub damage: f32,
    pub damage_type: DamageType,
    pub status_effects: Vec<StatusEffectData>,
}

/// Data for a status effect within an effect definition
#[derive(Debug, Clone)]
pub struct StatusEffectData {
    pub status_id: StatusId,
    pub intensity: f32,
    pub duration: f32,
    pub stack_behavior: StackBehavior,
}

/// Definition of a damage type's properties
#[derive(Debug, Clone)]
pub struct DamageTypeDefinition {
    pub name: String,
}

/// Registry that holds all effect and damage type definitions
/// This will be expanded with hot-reload support in later phases
#[derive(Resource, Default)]
pub struct EffectRegistry {
    pub effects: HashMap<EffectDefId, EffectDefinition>,
    pub damage_types: HashMap<DamageType, DamageTypeDefinition>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            effects: HashMap::new(),
            damage_types: HashMap::new(),
        };

        // Register default damage types
        registry.register_damage_type(DamageType::PHYSICAL, DamageTypeDefinition {
            name: "Physical".to_string(),
        });

        registry.register_damage_type(DamageType::MAGICAL, DamageTypeDefinition {
            name: "Magical".to_string(),
        });

        registry.register_damage_type(DamageType::TRUE, DamageTypeDefinition {
            name: "True".to_string(),
        });        registry
    }

    /// Initialize with basic effect definitions for testing
    pub fn with_basic_effects() -> Self {
        let mut registry = Self::new();

        // Basic player projectile
        registry.register_effect(EffectDefId(1), EffectDefinition {
            damage: 10.0,
            damage_type: DamageType::PHYSICAL,
            status_effects: vec![
                StatusEffectData {
                    status_id: StatusId::KNOCKBACK,
                    intensity: 200.0,
                    duration: 0.0, // Instant knockback
                    stack_behavior: StackBehavior::Stack,
                }
            ],
        });

        // Enemy bullet
        registry.register_effect(EffectDefId(2), EffectDefinition {
            damage: 15.0,
            damage_type: DamageType::PHYSICAL,
            status_effects: vec![], // No knockback
        });

        // Grenade explosion
        registry.register_effect(EffectDefId(3), EffectDefinition {
            damage: 100.0,
            damage_type: DamageType::PHYSICAL,
            status_effects: vec![
                StatusEffectData {
                    status_id: StatusId::KNOCKBACK,
                    intensity: 400.0,
                    duration: 0.0, // Instant knockback
                    stack_behavior: StackBehavior::Stack,
                }
            ],
        });

        // Contact damage
        registry.register_effect(EffectDefId(4), EffectDefinition {
            damage: 25.0,
            damage_type: DamageType::PHYSICAL,
            status_effects: vec![], // No knockback
        });

        registry
    }    pub fn register_effect(&mut self, id: EffectDefId, definition: EffectDefinition) {
        self.effects.insert(id, definition);
    }

    pub fn get_effect(&self, id: EffectDefId) -> Option<&EffectDefinition> {
        self.effects.get(&id)
    }

    pub fn register_damage_type(&mut self, id: DamageType, definition: DamageTypeDefinition) {
        self.damage_types.insert(id, definition);
    }

    pub fn get_damage_type(&self, id: DamageType) -> Option<&DamageTypeDefinition> {
        self.damage_types.get(&id)
    }


}
