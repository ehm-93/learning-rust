use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;

/// Trait that all scenes must implement
pub trait Scene: Send + Sync + 'static {
    /// Called when the scene is first created
    fn setup(&mut self, world: &mut World);

    /// Called every frame while the scene is active
    fn update(&mut self, world: &mut World);

    /// Called when the scene is being destroyed
    fn teardown(&mut self, world: &mut World);

    /// Returns the name of this scene for debugging
    fn name(&self) -> &'static str;

    /// Returns whether this scene should be paused when another scene is pushed on top
    fn pausable(&self) -> bool { true }

    /// Returns whether this scene should be transparent (other scenes visible underneath)
    fn transparent(&self) -> bool { false }
}

/// Component to mark entities as belonging to a specific scene
#[derive(Component)]
pub struct SceneEntity {
    pub scene_id: SceneId,
}

/// Unique identifier for scenes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(pub TypeId);

impl SceneId {
    pub fn of<T: Scene + ?Sized>() -> Self {
        Self(TypeId::of::<T>())
    }
}

/// Resource that manages the scene stack
#[derive(Resource)]
pub struct SceneManager {
    /// Stack of active scenes (top is current)
    scene_stack: Vec<Box<dyn Scene>>,
    /// Mapping of scene types to their IDs
    scene_ids: HashMap<TypeId, SceneId>,
    /// Pending scene transitions
    pending_transitions: Vec<SceneTransition>,
}

/// Types of scene transitions
pub enum SceneTransition {
    /// Replace all scenes with a new one
    Replace(Box<dyn Scene>),
    /// Push a new scene on top of the current one
    Push(Box<dyn Scene>),
    /// Pop the current scene
    Pop,
    /// Pop all scenes and push a new one
    Clear,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self {
            scene_stack: Vec::new(),
            scene_ids: HashMap::new(),
            pending_transitions: Vec::new(),
        }
    }
}

impl SceneManager {
    /// Create a new empty scene manager
    pub fn new() -> Self {
        Self {
            scene_stack: Vec::new(),
            scene_ids: HashMap::new(),
            pending_transitions: Vec::new(),
        }
    }
    /// Replace the current scene with a new one
    pub fn replace_scene<T: Scene>(&mut self, scene: T) {
        self.pending_transitions.push(SceneTransition::Replace(Box::new(scene)));
    }

    /// Push a new scene on top of the current one
    pub fn push_scene<T: Scene>(&mut self, scene: T) {
        self.pending_transitions.push(SceneTransition::Push(Box::new(scene)));
    }

    /// Pop the current scene
    pub fn pop_scene(&mut self) {
        self.pending_transitions.push(SceneTransition::Pop);
    }

    /// Clear all scenes
    pub fn clear_scenes(&mut self) {
        self.pending_transitions.push(SceneTransition::Clear);
    }

    /// Get the current active scene
    pub fn current_scene(&self) -> Option<&dyn Scene> {
        self.scene_stack.last().map(|s| s.as_ref())
    }

    /// Check if a specific scene type is in the stack
    pub fn has_scene<T: Scene>(&self) -> bool {
        let target_id = TypeId::of::<T>();
        self.scene_stack.iter().any(|scene| {
            // This is a bit hacky but works for our purposes
            std::ptr::eq(scene.as_ref() as *const dyn Scene as *const (),
                        &target_id as *const TypeId as *const ())
        })
    }

    /// Get the number of scenes in the stack
    pub fn scene_count(&self) -> usize {
        self.scene_stack.len()
    }
}

/// System to process pending scene transitions
pub fn process_scene_transitions(world: &mut World) {
    // We need to extract the SceneManager to avoid borrowing issues
    let mut scene_manager = world.remove_resource::<SceneManager>().unwrap_or_default();
    let transitions = std::mem::take(&mut scene_manager.pending_transitions);

    for transition in transitions {
        match transition {
            SceneTransition::Replace(new_scene) => {
                // Teardown current scene
                if let Some(mut current) = scene_manager.scene_stack.pop() {
                    current.teardown(world);
                    cleanup_scene_entities(world, SceneId::of::<dyn Scene>());
                }

                // Setup new scene
                let mut new_scene = new_scene;
                new_scene.setup(world);
                scene_manager.scene_stack.push(new_scene);
            },
            SceneTransition::Push(new_scene) => {
                // Setup new scene
                let mut new_scene = new_scene;
                new_scene.setup(world);
                scene_manager.scene_stack.push(new_scene);
            },
            SceneTransition::Pop => {
                if let Some(mut scene) = scene_manager.scene_stack.pop() {
                    scene.teardown(world);
                    cleanup_scene_entities(world, SceneId::of::<dyn Scene>());
                }
            },
            SceneTransition::Clear => {
                // Teardown all scenes
                while let Some(mut scene) = scene_manager.scene_stack.pop() {
                    scene.teardown(world);
                }
                cleanup_all_scene_entities(world);
            },
        }
    }

    // Put the SceneManager back
    world.insert_resource(scene_manager);
}

/// System to update the current scene
pub fn update_current_scene(world: &mut World) {
    // Similar pattern - extract, modify, put back
    let mut scene_manager = world.remove_resource::<SceneManager>().unwrap_or_default();

    if let Some(scene) = scene_manager.scene_stack.last_mut() {
        scene.update(world);
    }

    world.insert_resource(scene_manager);
}

/// System for updating scenes - should be added to Bevy's schedule
pub fn update_scenes(world: &mut World) {
    process_scene_transitions(world);
}

/// Helper function to cleanup entities belonging to a specific scene
fn cleanup_scene_entities(world: &mut World, scene_id: SceneId) {
    let entities_to_remove: Vec<Entity> = world
        .query::<(Entity, &SceneEntity)>()
        .iter(world)
        .filter_map(|(entity, scene_entity)| {
            if scene_entity.scene_id == scene_id {
                Some(entity)
            } else {
                None
            }
        })
        .collect();

    for entity in entities_to_remove {
        if let Ok(entity_ref) = world.get_entity_mut(entity) {
            entity_ref.despawn();
        }
    }
}

/// Helper function to cleanup all scene entities
fn cleanup_all_scene_entities(world: &mut World) {
    let entities_to_remove: Vec<Entity> = world
        .query::<(Entity, &SceneEntity)>()
        .iter(world)
        .map(|(entity, _)| entity)
        .collect();

    for entity in entities_to_remove {
        if let Ok(entity_ref) = world.get_entity_mut(entity) {
            entity_ref.despawn();
        }
    }
}

/// Helper trait for spawning entities with scene tracking
pub trait SceneEntityCommands {
    fn spawn_in_scene<T: Scene>(&mut self, bundle: impl Bundle) -> EntityCommands;
}

impl SceneEntityCommands for Commands<'_, '_> {
    fn spawn_in_scene<T: Scene>(&mut self, bundle: impl Bundle) -> EntityCommands {
        let mut entity = self.spawn(bundle);
        entity.insert(SceneEntity {
            scene_id: SceneId::of::<T>(),
        });
        entity
    }
}

/// Plugin to add scene management to the app
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneManager>()
            .add_systems(Update, (
                process_scene_transitions,
                update_current_scene,
            ).chain());
    }
}
