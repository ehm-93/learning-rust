use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashMap;

use super::effects::*;

/// Combat state component that replaces the old Health component
/// Contains all combat-related data including resistances and modifiers
#[derive(Component, Debug, Clone)]
pub struct CombatState {
    pub health: f32,
    pub max_health: f32,
    /// Damage multipliers by type (1.0 = normal, 0.5 = half damage, 2.0 = double damage, 0.0 = immune)
    pub damage_multipliers: HashMap<DamageType, f32>,
    /// Status effect multipliers (1.0 = normal, 0.5 = half effect, 2.0 = double effect, 0.0 = immune)
    pub status_multipliers: HashMap<StatusId, f32>,
}

impl CombatState {
    pub fn new(max_health: f32) -> Self {
        Self {
            health: max_health,
            max_health,
            damage_multipliers: HashMap::new(),
            status_multipliers: HashMap::new(),
        }
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// Get damage multiplier for a damage type
    /// 1.0 = normal damage, 0.5 = half damage, 2.0 = double damage, 0.0 = immune
    pub fn get_damage_multiplier(&self, damage_type: DamageType) -> f32 {
        self.damage_multipliers.get(&damage_type).copied().unwrap_or(1.0)
    }

    /// Get status effect multiplier
    /// 1.0 = normal effect, 0.5 = half effect, 2.0 = double effect, 0.0 = immune
    pub fn get_status_multiplier(&self, status_id: StatusId) -> f32 {
        self.status_multipliers.get(&status_id).copied().unwrap_or(1.0)
    }
}

/// Active status effect on an entity
#[derive(Component, Debug, Clone)]
pub struct StatusEffect {
    pub status_id: StatusId,
    pub intensity: f32,
    pub remaining_duration: f32,
    pub stack_behavior: StackBehavior,
}

/// System that resolves EffectEvents into specific damage/knockback/status events
/// This is the core of Phase 0.a - handles multi-hit resolution
pub fn resolve_effects(
    mut effect_events: EventReader<EffectEvent>,
    mut damage_events: EventWriter<DamageEvent>,
    mut status_events: EventWriter<StatusEvent>,
    effect_registry: Res<EffectRegistry>,
    combat_query: Query<&CombatState>,
    faction_query: Query<&Faction>,
) {
    // Group all effects by target for multi-hit resolution
    let mut effects_by_target: HashMap<Entity, Vec<(Entity, &EffectDefinition, Vec2)>> = HashMap::new();

    // Collect all effects this frame, grouped by target
    for effect_event in effect_events.read() {
        if let Some(definition) = effect_registry.get_effect(effect_event.effect_id) {
            for (target, hit_pos) in effect_event.targets.iter().zip(effect_event.hit_positions.iter()) {
                effects_by_target
                    .entry(*target)
                    .or_insert_with(Vec::new)
                    .push((effect_event.source, definition, *hit_pos));
            }
        }
    }

    // Process each target's accumulated effects
    for (target, effects) in effects_by_target {
        if let Ok(target_combat) = combat_query.get(target) {
            resolve_damage_for_target(target, &effects, target_combat, &effect_registry, &mut damage_events);
            resolve_status_for_target(target, &effects, target_combat, &mut status_events);
        }
    }
}

/// Resolve all damage effects for a single target
fn resolve_damage_for_target(
    target: Entity,
    effects: &[(Entity, &EffectDefinition, Vec2)],
    target_combat: &CombatState,
    effect_registry: &EffectRegistry,
    damage_events: &mut EventWriter<DamageEvent>,
) {
    // Group damage by type and source for proper resolution
    let mut damage_by_type: HashMap<DamageType, f32> = HashMap::new();
    let mut damage_sources: Vec<Entity> = Vec::new();

    for (source, definition, _hit_pos) in effects {
        let base_damage = definition.damage;
        if base_damage > 0.0 {
            // Accumulate damage by type (multi-hit same-frame: sum all damage)
            *damage_by_type.entry(definition.damage_type).or_insert(0.0) += base_damage;
            damage_sources.push(*source);
        }
    }

    // Apply resistances and emit damage events
    for (damage_type, total_damage) in damage_by_type {
        if total_damage > 0.0 {
            let final_damage = calculate_final_damage(total_damage, damage_type, target_combat, effect_registry);

            // Use first source for the damage event (could be improved later)
            let source = damage_sources.first().copied().unwrap_or(Entity::PLACEHOLDER);

            damage_events.send(DamageEvent {
                target,
                damage: final_damage,
                damage_type,
                source,
            });
        }
    }
}

/// Calculate final damage after multipliers
fn calculate_final_damage(
    base_damage: f32,
    damage_type: DamageType,
    target_combat: &CombatState,
    _effect_registry: &EffectRegistry,
) -> f32 {
    let multiplier = target_combat.get_damage_multiplier(damage_type);
    // Simple multiplication: 1.0 = normal, 0.5 = half damage, 2.0 = double damage, 0.0 = immune
    base_damage * multiplier
}



/// Resolve all status effects for a single target
fn resolve_status_for_target(
    target: Entity,
    effects: &[(Entity, &EffectDefinition, Vec2)],
    target_combat: &CombatState,
    status_events: &mut EventWriter<StatusEvent>,
) {
    // Group status effects by ID for proper stacking resolution
    let mut status_by_id: HashMap<StatusId, Vec<(f32, f32, Entity)>> = HashMap::new();

    for (source, definition, _hit_pos) in effects {
        for status_data in &definition.status_effects {
            status_by_id
                .entry(status_data.status_id)
                .or_insert_with(Vec::new)
                .push((status_data.intensity, status_data.duration, *source));
        }
    }

    // Resolve each status type according to its stacking behavior
    for (status_id, status_instances) in status_by_id {
        if let Some((intensity, duration, source)) = resolve_status_stacking(status_instances, status_id) {
            // Apply status multiplier
            let multiplier = target_combat.get_status_multiplier(status_id);
            let final_intensity = intensity * multiplier;
            let final_duration = duration * multiplier;

            if final_intensity > 0.0 && final_duration > 0.0 {
                status_events.send(StatusEvent {
                    target,
                    status_id,
                    intensity: final_intensity,
                    duration: final_duration,
                    source,
                });
            }
        }
    }
}

/// Resolve how multiple status effects of the same type stack
fn resolve_status_stacking(
    instances: Vec<(f32, f32, Entity)>,
    status_id: StatusId,
) -> Option<(f32, f32, Entity)> {
    if instances.is_empty() {
        return None;
    }

    // For now, use a simple default stacking behavior
    // This will be data-driven when we add status effect definitions
    let stack_behavior = get_default_stack_behavior(status_id);

    match stack_behavior {
        StackBehavior::Replace => {
            // Use the last applied effect
            instances.last().copied()
        }
        StackBehavior::Stack => {
            // Sum all intensities and durations
            let total_intensity: f32 = instances.iter().map(|(i, _, _)| i).sum();
            let total_duration: f32 = instances.iter().map(|(_, d, _)| d).sum();
            let source = instances.first().unwrap().2;
            Some((total_intensity, total_duration, source))
        }
        StackBehavior::RefreshHighest => {
            // Highest intensity, longest duration
            let max_intensity = instances.iter().map(|(i, _, _)| *i).fold(0.0, f32::max);
            let max_duration = instances.iter().map(|(_, d, _)| *d).fold(0.0, f32::max);
            let source = instances.iter()
                .find(|(i, _, _)| *i == max_intensity)
                .unwrap_or(&instances[0]).2;
            Some((max_intensity, max_duration, source))
        }
    }
}

/// Get default stacking behavior for status effects
/// This is temporary - will be data-driven later
fn get_default_stack_behavior(status_id: StatusId) -> StackBehavior {
    match status_id {
        StatusId::SLOW => StackBehavior::RefreshHighest,
        StatusId::STUN => StackBehavior::RefreshHighest,
        StatusId::DAMAGE_OVER_TIME => StackBehavior::Stack,
        StatusId::KNOCKBACK => StackBehavior::Stack, // Average knockback forces
        _ => StackBehavior::Replace,
    }
}

/// System that applies damage events to combat state
pub fn apply_damage(
    mut damage_events: EventReader<DamageEvent>,
    mut combat_query: Query<&mut CombatState>,
) {
    for damage_event in damage_events.read() {
        if let Ok(mut combat_state) = combat_query.get_mut(damage_event.target) {
            combat_state.health = (combat_state.health - damage_event.damage).max(0.0);
        }
    }
}



/// System that applies status effects to entities
pub fn apply_status_effects(
    mut status_events: EventReader<StatusEvent>,
    mut commands: Commands,
    mut velocity_query: Query<&mut Velocity>,
    entity_query: Query<Entity>,
) {
    for status_event in status_events.read() {
        if entity_query.contains(status_event.target) {
            // Handle instant knockback effects
            if status_event.status_id == StatusId::KNOCKBACK && status_event.duration == 0.0 {
                // Apply instant knockback to velocity
                if let Ok(mut velocity) = velocity_query.get_mut(status_event.target) {
                    // For now, simple knockback in X direction (will be improved later with proper directions)
                    velocity.linvel += Vec2::new(status_event.intensity, 0.0);
                }
            } else {
                // Add persistent status effect component
                commands.entity(status_event.target).insert(StatusEffect {
                    status_id: status_event.status_id,
                    intensity: status_event.intensity,
                    remaining_duration: status_event.duration,
                    stack_behavior: get_default_stack_behavior(status_event.status_id),
                });
            }
        }
    }
}

/// System that ticks down status effect durations and removes expired ones
pub fn tick_status_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut status_query: Query<(Entity, &mut StatusEffect)>,
) {
    for (entity, mut status) in status_query.iter_mut() {
        status.remaining_duration -= time.delta_secs();

        if status.remaining_duration <= 0.0 {
            commands.entity(entity).remove::<StatusEffect>();
        }
    }
}

/// System that cleans up dead entities (replaces the old cleanup_dead_entities)
pub fn cleanup_dead_entities(
    mut commands: Commands,
    combat_query: Query<(Entity, &CombatState)>,
    player_query: Query<&crate::player::Player>,
    enemy_query: Query<&crate::components::Enemy>,
    mut game_state: ResMut<crate::resources::GameState>,
) {
    for (entity, combat_state) in combat_query.iter() {
        if combat_state.is_dead() {
            // Check if it's the player
            if player_query.contains(entity) {
                *game_state = crate::resources::GameState::GameOver;
                continue; // Don't despawn player
            }

            // Award points for enemies (keeping existing logic for now)
            if let Ok(enemy) = enemy_query.get(entity) {
                let _points = match enemy.archetype {
                    crate::components::EnemyArchetype::SmallMelee => 10,
                    crate::components::EnemyArchetype::BigMelee => 50,
                    crate::components::EnemyArchetype::Shotgunner => 30,
                    crate::components::EnemyArchetype::Sniper => 75,
                    crate::components::EnemyArchetype::MachineGunner => 40,
                };
                // TODO: Add points to game state
            }

            commands.entity(entity).despawn();
        }
    }
}
