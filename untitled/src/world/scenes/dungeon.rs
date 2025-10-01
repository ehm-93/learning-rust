use bevy::prelude::*;

use crate::{
    resources::*,
    world::dungeon,
};
use super::manager::{Scene, SceneEntityCommands};

/// Dungeon scene implementation
pub struct DungeonScene {
    pub depth: u32,
    pub modifiers: Vec<String>, // Applied modifiers for this run
}

impl DungeonScene {
    pub fn new(depth: u32) -> Self {
        Self {
            depth,
            modifiers: Vec::new(),
        }
    }

    pub fn with_modifiers(depth: u32, modifiers: Vec<String>) -> Self {
        Self {
            depth,
            modifiers,
        }
    }
}

impl Scene for DungeonScene {
    fn setup(&mut self, world: &mut World) {
        info!("Setting up Dungeon scene at depth {}", self.depth);

        // Set game mode to dungeon
        if let Some(mut game_mode) = world.get_resource_mut::<GameMode>() {
            *game_mode = GameMode::Dungeon;
        }

        // Spawn a basic dungeon marker for now
        world.spawn((
            Transform::from_translation(Vec3::ZERO),
            Visibility::default(),
        ));

        // TODO: Apply modifiers to the dungeon
        if !self.modifiers.is_empty() {
            info!("Applying modifiers to dungeon: {:?}", self.modifiers);
        }
    }

    fn update(&mut self, _world: &mut World) {
        // Handle dungeon-specific updates
        // Could include enemy spawning, time limits, environmental effects, etc.
    }

    fn teardown(&mut self, _world: &mut World) {
        info!("Tearing down Dungeon scene (depth {})", self.depth);

        // Dungeon-specific cleanup
        // Scene entities will be automatically cleaned up by the scene manager
    }

    fn name(&self) -> &'static str {
        "Dungeon"
    }

    fn pausable(&self) -> bool {
        true
    }

    fn transparent(&self) -> bool {
        false
    }
}

/// Setup the dungeon scene
fn setup_dungeon_scene(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    depth: u32,
) {
    // Use the existing dungeon generation but with scene entity tracking
    dungeon::generate_dungeon_rooms_in_scene::<DungeonScene>(commands, meshes, materials, depth);

    // Add depth indicator
    commands.spawn_in_scene::<DungeonScene>((
        Text2d::new(format!("Dungeon Depth: {}", depth)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(-350.0, 250.0, 1.0)),
    ));
}
