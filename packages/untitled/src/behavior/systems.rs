use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::behavior::BehaviorComponent;

/// System to call on_update for all behaviors
/// Uses exclusive world access to allow behaviors to modify any entity
pub fn update_behaviors(world: &mut World) {
    let dt = world.resource::<Time>().delta().as_secs_f32();

    // Get all entities with BehaviorComponent
    let entities: Vec<Entity> = world
        .query::<Entity>()
        .iter(world)
        .filter(|e| world.get::<BehaviorComponent>(*e).is_some())
        .collect();

    // Update each behavior component
    // We need to remove the component, call on_update, then re-insert it
    // to avoid borrow checker issues with World
    for entity in entities {
        if let Some(mut behavior_comp) = world.entity_mut(entity).take::<BehaviorComponent>() {
            behavior_comp.on_update(world, dt);
            // Check if entity still exists (behavior might have despawned it)
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.insert(behavior_comp);
            }
        }
    }
}

/// System to handle collision events and call collision hooks
pub fn handle_collisions(world: &mut World) {
    // Read and drain collision events
    let events: Vec<CollisionEvent> = {
        let mut collision_events = world.resource_mut::<Events<CollisionEvent>>();
        collision_events.drain().collect()
    };

    // Process each collision event
    for event in events {
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {
                // Call on_collision_enter for entity 1
                if let Some(mut behavior_comp) = world.entity_mut(e1).take::<BehaviorComponent>() {
                    behavior_comp.on_collision_enter(world, e2);
                    if let Ok(mut entity_mut) = world.get_entity_mut(e1) {
                        entity_mut.insert(behavior_comp);
                    }
                }
                // Call on_collision_enter for entity 2
                if let Some(mut behavior_comp) = world.entity_mut(e2).take::<BehaviorComponent>() {
                    behavior_comp.on_collision_enter(world, e1);
                    if let Ok(mut entity_mut) = world.get_entity_mut(e2) {
                        entity_mut.insert(behavior_comp);
                    }
                }
            },
            CollisionEvent::Stopped(e1, e2, _flags) => {
                // Call on_collision_exit for entity 1
                if let Some(mut behavior_comp) = world.entity_mut(e1).take::<BehaviorComponent>() {
                    behavior_comp.on_collision_exit(world, e2);
                    if let Ok(mut entity_mut) = world.get_entity_mut(e1) {
                        entity_mut.insert(behavior_comp);
                    }
                }
                // Call on_collision_exit for entity 2
                if let Some(mut behavior_comp) = world.entity_mut(e2).take::<BehaviorComponent>() {
                    behavior_comp.on_collision_exit(world, e1);
                    if let Ok(mut entity_mut) = world.get_entity_mut(e2) {
                        entity_mut.insert(behavior_comp);
                    }
                }
            },
        }
    }
}

/// System to call on_spawn for newly added behavior components
pub fn spawn_behaviors(world: &mut World) {
    // Query for entities with newly added BehaviorComponent
    let new_behaviors: Vec<Entity> = world
        .query_filtered::<Entity, Added<BehaviorComponent>>()
        .iter(world)
        .collect();

    // Call on_spawn for each new behavior
    for entity in new_behaviors {
        if let Some(mut behavior_comp) = world.entity_mut(entity).take::<BehaviorComponent>() {
            info!("Spawning behaviors for entity {:?}", entity);
            behavior_comp.on_spawn(world);
            // Check if entity still exists (behavior might have despawned it in on_spawn)
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.insert(behavior_comp);
            }
        }
    }
}

/// System to track entities about to be despawned
/// Note: Currently simplified - would need more complex tracking for proper despawn handling
pub fn despawn_behaviors(_world: &mut World) {
    // Despawn tracking is complex - would need a separate marker or queue system
    // Skipping for now as it requires architectural changes
}
