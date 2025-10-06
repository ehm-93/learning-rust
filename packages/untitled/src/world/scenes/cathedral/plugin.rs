use bevy::prelude::*;
use crate::world::states::WorldState;
use super::{resources::*, systems::*};

/// Cathedral plugin that manages the central hub and dungeon descent system
pub struct CathedralPlugin;

impl Plugin for CathedralPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add resources
            .init_resource::<ModifierSystem>()
            .init_resource::<ProgressionState>()
            .init_resource::<CathedralState>()
            .init_resource::<crate::world::states::CathedralConfig>()

            // Scene lifecycle systems
            .add_systems(OnEnter(WorldState::Cathedral), setup_cathedral_scene)
            .add_systems(OnExit(WorldState::Cathedral), teardown_cathedral_scene)
            .add_systems(
                FixedUpdate,
                (
                    handle_portal_interaction_events.run_if(in_state(WorldState::Cathedral)),
                    handle_portal_activation.run_if(in_state(WorldState::Cathedral)),
                    update_portal_displays.run_if(in_state(WorldState::Cathedral)),
                )
            );
    }
}
