use bevy::prelude::*;

use crate::world::chunks::ChunkingState;

use super::*;

/// Plugin for the Fog of War system
///
/// Registers FOW resources and systems with appropriate scheduling:
/// - `FixedUpdate`: Load/unload, task spawning/polling, lerping (budgeted, deterministic)
/// - `Update`: Drawing (visual smoothness, frame-rate dependent)
pub struct FowPlugin;

impl Plugin for FowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize resources
            .init_resource::<FowTaskSet>()
            // FixedUpdate: Core FOW logic (deterministic, budgeted)
            .add_systems(
                FixedUpdate,
                (
                    load_fow_chunks,
                    unload_fow_chunks,
                    spawn_fow_calculation_tasks,
                    poll_fow_calculation_tasks,
                    lerp_fow_vision,
                )
                    .chain()
                    .run_if(in_state(ChunkingState::Enabled)),
            )
            // Update: Visual rendering (smooth, frame-rate dependent)
            .add_systems(
                Update,
                draw_fow.run_if(in_state(ChunkingState::Enabled)),
            );
    }
}
