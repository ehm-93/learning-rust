pub mod components;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

use crate::world::states::WorldState;

/// Plugin for the dungeon scene
pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<resources::DungeonState>()

            // Add systems for dungeon state transitions
            .add_systems(OnEnter(WorldState::Dungeon), systems::setup_dungeon_scene)
            .add_systems(OnExit(WorldState::Dungeon), systems::teardown_dungeon_scene)

            // Add systems that run while in dungeon
            .add_systems(Update, (
                systems::handle_dungeon_portal_interactions,
            ).run_if(in_state(WorldState::Dungeon)));
    }
}
