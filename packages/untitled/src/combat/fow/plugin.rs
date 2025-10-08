use bevy::prelude::*;

use crate::world::chunks::ChunkingState;

use super::*;

pub struct FowPlugin;

impl Plugin for FowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                load_fow_chunks,
                unload_fow_chunks,
                update_fow_chunks,
                update_fow_chunks_continuous,
                draw_fow,
            ).run_if(in_state(ChunkingState::Enabled))
        );
    }
}
