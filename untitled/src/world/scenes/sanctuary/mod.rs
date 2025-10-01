pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
use crate::world::states::WorldState;

/// Sanctuary plugin that manages the safe zone with healing, vendors, and upgrades
pub struct SanctuaryPlugin;

impl Plugin for SanctuaryPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add resources
            .init_resource::<resources::SanctuaryState>()

            // Add systems for sanctuary state transitions
            .add_systems(OnEnter(WorldState::Sanctuary), (
                systems::update_sanctuary_depth_on_enter,
                systems::setup_sanctuary_scene,
            ).chain())
            .add_systems(OnExit(WorldState::Sanctuary), systems::teardown_sanctuary_scene)

            // Add systems that run while in sanctuary
            .add_systems(Update, (
                systems::handle_sanctuary_portal_interactions,
            ).run_if(in_state(WorldState::Sanctuary)));
    }
}
