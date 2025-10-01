use bevy::prelude::*;
use std::sync::Arc;
use rand::rng;
use crate::{
    world::{Interactable, InteractableHighlight, InteractionCallback},
    player::PlayerSpawner,
    ui::tooltip::Tooltip,
};
use super::{
    components::*,
    resources::*,
};
use crate::world::scenes::manager::{Scene, SceneEntity, SceneId};

/// Cathedral scene implementation
pub struct CathedralScene {
    pub state: CathedralState,
    pub modifier_system: ModifierSystem,
    pub progression_state: ProgressionState,
}

impl Default for CathedralScene {
    fn default() -> Self {
        Self {
            state: CathedralState::default(),
            modifier_system: ModifierSystem::new(),
            progression_state: ProgressionState::default(),
        }
    }
}

impl CathedralScene {
    /// Create a new cathedral scene
    pub fn new() -> Self {
        Self {
            state: CathedralState::default(),
            modifier_system: ModifierSystem::new(),
            progression_state: ProgressionState::default(),
        }
    }

    /// Spawn Cathedral-specific elements with proper scene tracking
    fn spawn_cathedral_elements(&self, world: &mut World) {
        let portal_positions = [
            Vec3::new(-200.0, 0.0, 0.0),  // Left portal
            Vec3::new(0.0, 0.0, 0.0),     // Center portal
            Vec3::new(200.0, 0.0, 0.0),   // Right portal
        ];

        let portal_colors = [
            Color::srgb(0.6, 0.3, 0.8), // Dark purple
            Color::srgb(0.8, 0.4, 1.0), // Bright purple
            Color::srgb(0.4, 0.2, 0.6), // Deep purple
        ];

        let portal_ids = [PortalId::Left, PortalId::Center, PortalId::Right];

        // Pre-create all assets to avoid borrow checker issues
        let portal_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Rectangle::new(80.0, 120.0))
        };

        let portal_materials: Vec<_> = {
            let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
            portal_colors.iter().map(|&color| materials.add(color)).collect()
        };

        let floor_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Rectangle::new(800.0, 100.0))
        };

        let floor_material = {
            let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
            materials.add(Color::srgb(0.2, 0.2, 0.2))
        };

        // Create three portal archways
        // Initialize portal configurations
        let mut rng = rng();
        let current_depth = 1; // Start at depth 1
        let mut modifier_system = self.modifier_system.clone();
        modifier_system.generate_portal_modifiers(current_depth, &mut rng);

        for (i, (&position, &portal_id)) in portal_positions.iter().zip(portal_ids.iter()).enumerate() {
            // Create portal callback for transitions
            // The callback is required but the actual transition logic is handled by the portal_interaction_events system
            let portal_callback: InteractionCallback = Arc::new(move |_context| {
                // No-op - the event system handles portal transitions
            });

            // Get modifiers for this portal
            let portal_modifiers = modifier_system.get_portal_modifiers(current_depth, portal_id);

            // Create tooltip content
            let mut tooltip_content = format!("{:?} Portal\nDepth {}\n\n", portal_id, current_depth);

            if portal_modifiers.is_empty() {
                tooltip_content.push_str("No Modifiers");
            } else {
                tooltip_content.push_str("Modifiers:\n");
                for modifier in &portal_modifiers {
                    tooltip_content.push_str(&format!("• {}\n", modifier.display_name()));
                }
            }

            // Create portal entity with scene tracking and tooltip
            world.spawn((
                Mesh2d(portal_mesh.clone()),
                MeshMaterial2d(portal_materials[i].clone()),
                Transform::from_translation(position),
                Portal {
                    id: portal_id,
                    depth: current_depth,
                    modifiers: portal_modifiers.clone(),
                },
                Interactable::new(
                    format!("portal_{:?}", portal_id), // id
                    format!("{:?} Portal", portal_id), // display name
                    portal_callback
                ),
                InteractableHighlight::with_radius(1.4), // Use default colors with custom radius
                Tooltip::new(tooltip_content)
                    .with_offset(Vec3::new(0.0, 80.0, 10.0)) // Show above the portal
                    .with_font_size(16.0)
                    .with_background_color(Color::srgba(0.1, 0.0, 0.2, 0.9)) // Dark purple background
                    .with_text_color(Color::WHITE)
                    .with_max_width(250.0)
                    .with_padding(12.0),
                SceneEntity {
                    scene_id: SceneId::of::<CathedralScene>(),
                },
            ));
        }

        // Add some basic Cathedral decoration (simple floor) with scene tracking
        world.spawn((
            Mesh2d(floor_mesh),
            MeshMaterial2d(floor_material),
            Transform::from_translation(Vec3::new(0.0, -200.0, -1.0)),
            SceneEntity {
                scene_id: SceneId::of::<CathedralScene>(),
            },
        ));

        // Add Cathedral title text with scene tracking
        world.spawn((
            Text2d::new("The Cathedral"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(0.0, 150.0, 1.0)),
            SceneEntity {
                scene_id: SceneId::of::<CathedralScene>(),
            },
        ));

        info!("Cathedral scene elements spawned with scene tracking");
    }
}

impl Scene for CathedralScene {
    fn setup(&mut self, world: &mut World) {
        info!("Setting up Cathedral scene");

        // Mark Cathedral as active
        self.state.is_active = true;

        // Insert resources
        world.insert_resource(self.state.clone());
        world.insert_resource(self.modifier_system.clone());
        world.insert_resource(self.progression_state.clone());

        // Spawn camera with scene tracking
        world.spawn((
            Camera2d,
            crate::components::MainCamera,
            SceneEntity {
                scene_id: SceneId::of::<CathedralScene>(),
            },
        ));

        // Spawn player using the player spawning service
        PlayerSpawner::spawn_in_scene::<CathedralScene>(world, Vec3::new(0.0, 0.0, 0.0));        // Spawn Cathedral scene elements (portals, decorations, etc.) with scene tracking
        self.spawn_cathedral_elements(world);

        // Initialize portals with modifiers
        let mut rng = rand::rng();
        self.modifier_system.generate_portal_modifiers(1, &mut rng);
    }

    fn update(&mut self, _world: &mut World) {
        // Handle any per-frame updates
        // The interaction systems and portal updates are handled by the main game loop
        // This could be used for animations, time-based effects, etc.
    }

    fn teardown(&mut self, world: &mut World) {
        info!("Tearing down Cathedral scene");

        // Mark Cathedral as inactive
        self.state.is_active = false;

        // Remove resources
        world.remove_resource::<CathedralState>();
        world.remove_resource::<ModifierSystem>();
        world.remove_resource::<ProgressionState>();

        // Scene entities are automatically cleaned up by the scene manager
        // via the SceneEntity component that was added during spawning
    }

    fn name(&self) -> &'static str {
        "Cathedral"
    }

    fn pausable(&self) -> bool {
        true
    }

    fn transparent(&self) -> bool {
        false
    }
}
