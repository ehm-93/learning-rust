pub mod components;
pub mod resources;
pub mod systems;
pub mod scene;

use bevy::prelude::*;

/// Cathedral plugin that manages the central hub and dungeon descent system
pub struct CathedralPlugin;

impl Plugin for CathedralPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add resources
            .init_resource::<resources::CathedralState>()
            .init_resource::<resources::ModifierSystem>()
            .init_resource::<resources::ProgressionState>()
            // Add startup systems
            .add_systems(Startup, (
                systems::setup_cathedral_scene,
                systems::initialize_portals,
            ))
            // Add update systems (only run in Cathedral mode)
            .add_systems(Update, (
                systems::handle_portal_interaction.run_if(resource_equals(crate::resources::GameMode::Cathedral)),
                systems::handle_portal_interaction_events.run_if(resource_equals(crate::resources::GameMode::Cathedral)),
                systems::update_portal_displays.run_if(resource_equals(crate::resources::GameMode::Cathedral)),
            ));
    }
}

// Re-export commonly used types
pub use components::*;
pub use resources::*;
pub use systems::*;
pub use scene::*;
