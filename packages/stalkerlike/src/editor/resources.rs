use bevy::prelude::*;

/// Input state for editor camera control
#[derive(Resource, Default)]
pub struct EditorMouseMotion {
    pub delta: Vec2,
}
