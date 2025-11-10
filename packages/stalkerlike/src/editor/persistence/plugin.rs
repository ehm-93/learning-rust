//! Persistence plugin for scene save/load functionality

use bevy::prelude::*;

use super::events::{NewFileEvent, OpenFileEvent, SaveEvent, SaveAsEvent};
use super::systems::{CurrentFile, save_scene_system, load_scene_system, mark_scene_dirty, handle_new_file, handle_save, handle_open_file, handle_save_as};

/// Plugin for scene persistence (save, load, file operations)
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app
            // Events
            .add_event::<NewFileEvent>()
            .add_event::<OpenFileEvent>()
            .add_event::<SaveEvent>()
            .add_event::<SaveAsEvent>()

            // Resources
            .init_resource::<CurrentFile>()

            // Update systems
            .add_systems(Update, (
                save_scene_system,
                load_scene_system,
                mark_scene_dirty,
                handle_new_file,
                handle_save,
                handle_open_file,
                handle_save_as,
            ));
    }
}
