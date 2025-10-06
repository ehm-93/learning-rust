pub mod resources;
pub mod components;

mod collision;
mod systems;
pub mod terrain;

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
            .add_plugins(terrain::TerrainChunkPlugin)

            // Add systems that run while in dungeon
            .add_systems(FixedUpdate, (
                systems::handle_dungeon_portal_interactions,
            ).run_if(in_state(WorldState::Dungeon)));
    }
}
