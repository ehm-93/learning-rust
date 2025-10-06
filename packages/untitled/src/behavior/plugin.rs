use bevy::prelude::*;
use crate::behavior::{systems::*, BehaviorRegistry};

/// Plugin that adds the behavior system to a Bevy app
pub struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        // Add the behavior registry resource
        app.insert_resource(BehaviorRegistry::new());

        // Add behavior systems
        app.add_systems(FixedUpdate, (
            spawn_behaviors,
            update_behaviors,
            handle_collisions,
            process_behavior_spawn_queue,
        ).chain()); // Chain to ensure proper execution order

        app.add_systems(Last, despawn_behaviors);
    }
}
