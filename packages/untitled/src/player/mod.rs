use bevy::prelude::*;

pub mod components;
pub mod systems;
pub mod resources;
pub mod actions;
pub mod input;

pub use components::*;
pub use systems::*;
pub use resources::*;
pub use actions::*;
pub use input::*;

/// Plugin that handles all player-related functionality
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register player components
            .register_type::<Player>()
            .register_type::<Dash>()
            .register_type::<GrenadeThrower>()

            // Add player resources
            .insert_resource(FireTimer::default())
            .insert_resource(PlayerConfig::default())
            .insert_resource(PlayerInputBindings::default())
            .insert_resource(CameraZoom::default())

            // Add player action events
            .add_event::<PlayerActionEvent>()

            // Add input processing system first
            .add_systems(PreUpdate, player_input_system)

            // Add player systems
            .add_systems(Update, (
                player_movement,
                shoot_projectiles,
                throw_grenades,
                camera_follow,
                handle_camera_zoom,
            ));
    }
}
