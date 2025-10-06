use bevy::prelude::*;
use crate::behavior::{
    atomic::*, systems::*, BehaviorRegistry, CompositeBehavior
};

/// Plugin that adds the behavior system to a Bevy app
pub struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        // Add the behavior registry resource
        app.insert_resource(BehaviorRegistry::new());

        // Add behavior systems
        app.add_systems(Update, (
            spawn_behaviors,
            update_behaviors,
            handle_collisions,
            process_behavior_spawn_queue,
        ).chain()); // Chain to ensure proper execution order

        app.add_systems(Last, despawn_behaviors);

        // Register startup system to register default behaviors
        app.add_systems(Startup, register_default_behaviors);
    }
}

/// System to register default atomic behaviors and some composite examples
fn register_default_behaviors(mut registry: ResMut<BehaviorRegistry>) {
    // Register atomic behaviors
    registry.register("movement", movement_def);
    registry.register("homing", homing_def);
    registry.register("on_hit", on_hit_def);
    registry.register("damage_area", damage_area_def);
    registry.register("timeout", timeout_def);
    registry.register("projectile", projectile_def);
    registry.register("basic_projectile", |params| {
        Box::new(CompositeBehavior::new(
        vec![
                movement_def(params.clone()),
                projectile_def(params.clone()),
                on_hit_def(params.clone()),
            ]
        ))
    });

    // Explosive projectile = movement + projectile + on_hit
    registry.register("explosive_projectile", |params| {
        Box::new(CompositeBehavior::new(
            vec![
                movement_def(params.clone()),
                projectile_def(params.clone()),
                on_hit_def(params.clone()),
            ]
        ))
    });

    // Homing missile = movement + homing + projectile + on_hit
    registry.register("homing_missile", |params| {
        Box::new(CompositeBehavior::new(
        vec![
                movement_def(params.clone()),
                homing_def(params.clone()),
                projectile_def(params.clone()),
                on_hit_def(params.clone()),
            ]
        ))
    });

    // Explosion = damage_area + timeout
    registry.register("explosion", |params| {
        Box::new(CompositeBehavior::new(
            vec![
                damage_area_def(params.clone()),
                timeout_def(params.clone()),
            ]
        ))
    });

    info!("Registered {} behaviors", registry.list_behaviors().len());
    info!("Available behaviors: {:?}", registry.list_behaviors());
}
