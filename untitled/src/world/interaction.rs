use bevy::prelude::*;
use std::sync::Arc;
use crate::{
    components::MainCamera,
    player::Player,
};

/// Context passed to interaction callbacks
#[derive(Clone)]
pub struct InteractionContext {
    /// Entity that was interacted with
    pub target_entity: Entity,
    /// Entity that initiated the interaction (usually player)
    pub source_entity: Entity,
}

/// Callback function type for interactions
pub type InteractionCallback = Arc<dyn Fn(&InteractionContext) + Send + Sync>;

/// Simple interaction type with callback-based behavior
#[derive(Clone)]
pub struct InteractionType {
    /// Unique identifier for this interaction type
    pub id: String,
    /// Human-readable description (for debugging/logging)
    pub description: String,
    /// The callback function to execute when interacted with
    pub on_interact: InteractionCallback,
}

/// Component for objects that can be interacted with by the player
#[derive(Component)]
pub struct Interactable {
    /// The type of interaction this object provides
    pub interaction_type: InteractionType,
    /// Display name shown to the player
    pub display_name: String,
    /// Whether this interactable is currently available
    pub is_enabled: bool,
    /// Distance at which interaction becomes available
    pub interaction_range: f32,
    /// Cooldown timer for repeated interactions
    pub cooldown: Option<Timer>,
}

impl Interactable {
    /// Create a new interactable with callback
    pub fn new(id: impl Into<String>, display_name: impl Into<String>, callback: InteractionCallback) -> Self {
        let display_string = display_name.into();
        Self {
            interaction_type: InteractionType {
                id: id.into(),
                description: format!("Interactable: {}", display_string),
                on_interact: callback,
            },
            display_name: display_string,
            is_enabled: true,
            interaction_range: 100.0,
            cooldown: None,
        }
    }

    /// Builder method to set interaction range
    pub fn with_range(mut self, range: f32) -> Self {
        self.interaction_range = range;
        self
    }

    /// Builder method to set cooldown
    pub fn with_cooldown(mut self, seconds: f32) -> Self {
        self.cooldown = Some(Timer::from_seconds(seconds, TimerMode::Once));
        self
    }

    /// Builder method to set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.is_enabled = enabled;
        self
    }

    /// Check if the interactable is currently available for use
    pub fn can_interact(&self) -> bool {
        if !self.is_enabled {
            return false;
        }

        if let Some(cooldown) = &self.cooldown {
            return cooldown.finished();
        }

        true
    }

    /// Reset cooldown timer if it exists
    pub fn trigger_cooldown(&mut self) {
        if let Some(cooldown) = &mut self.cooldown {
            cooldown.reset();
        }
    }

    /// Update cooldown timers
    pub fn update_cooldown(&mut self, delta: std::time::Duration) {
        if let Some(cooldown) = &mut self.cooldown {
            cooldown.tick(delta);
        }
    }
}

/// Component to track hover/highlight state for interactables
#[derive(Component)]
pub struct InteractableHighlight {
    /// Whether the interactable is currently being hovered
    pub is_hovered: bool,
    /// Whether the player is within interaction range
    pub in_range: bool,
    /// Entity of the halo effect (if spawned)
    pub halo_entity: Option<Entity>,
    /// Halo color when player is in range
    pub in_range_color: Color,
    /// Halo color when player is out of range
    pub out_of_range_color: Color,
    /// Halo radius multiplier (relative to object size)
    pub halo_radius: f32,
    /// Halo opacity when fully visible
    pub halo_opacity: f32,
}

impl Default for InteractableHighlight {
    fn default() -> Self {
        Self {
            is_hovered: false,
            in_range: false,
            halo_entity: None,
            in_range_color: Color::srgba(0.2, 1.0, 0.3, 0.6), // Green glow for in-range
            out_of_range_color: Color::srgba(1.0, 0.8, 0.2, 0.4), // Yellow/orange glow for out-of-range
            halo_radius: 1.3,
            halo_opacity: 0.6,
        }
    }
}

impl InteractableHighlight {
    /// Create a new highlight component with custom halo settings
    pub fn new(in_range_color: Color, out_of_range_color: Color, halo_radius: f32) -> Self {
        Self {
            in_range_color,
            out_of_range_color,
            halo_radius,
            ..Default::default()
        }
    }

    /// Create a new highlight with default colors but custom radius
    pub fn with_radius(halo_radius: f32) -> Self {
        Self {
            halo_radius,
            ..Default::default()
        }
    }

    /// Get the current halo color based on range state
    pub fn current_color(&self) -> Color {
        if self.in_range {
            self.in_range_color
        } else {
            self.out_of_range_color
        }
    }
}

/// Marker component for the currently hovered interactable
#[derive(Component)]
pub struct HoveredInteractable;

/// Event fired when an interactable is activated
#[derive(Event)]
pub struct InteractionEvent {
    /// Entity that was interacted with
    pub target_entity: Entity,
    /// Entity that initiated the interaction (usually player)
    pub source_entity: Entity,
    /// Type of interaction that occurred
    pub interaction_type: InteractionType,
}

/// System to update cooldown timers on all interactables
pub fn update_interactable_cooldowns(
    time: Res<Time>,
    mut interactables: Query<&mut Interactable>,
) {
    for mut interactable in interactables.iter_mut() {
        interactable.update_cooldown(time.delta());
    }
}

/// System to handle basic interaction detection using PlayerActionEvents
/// This integrates with the player action system for consistent input handling
pub fn handle_basic_interactions(
    mut action_events: EventReader<crate::player::actions::PlayerActionEvent>,
    player_query: Query<Entity, With<Player>>,
    mut interactables: Query<&mut Interactable>,
    hovered_query: Query<Entity, (With<HoveredInteractable>, With<InteractableHighlight>)>,
    mut interaction_events: EventWriter<InteractionEvent>,
) {
    // Check for interact action events
    let interact_pressed = action_events.read().any(|event| {
        matches!(event.action, crate::player::actions::PlayerAction::Interact) && event.just_started()
    });

    if !interact_pressed {
        return;
    }

    let Ok(player_entity) = player_query.single() else {
        return;
    };

    // Find the interactable that has the HoveredInteractable marker component
    if let Ok(target_entity) = hovered_query.single() {
        // Get the interactable component to check if we can interact
        if let Ok(interactable) = interactables.get(target_entity) {
            if !interactable.can_interact() {
                return;
            }
        } else {
            return;
        }
        if let Ok(mut interactable) = interactables.get_mut(target_entity) {
            interactable.trigger_cooldown();

            // Execute callback
            let context = InteractionContext {
                target_entity,
                source_entity: player_entity,
            };
            (interactable.interaction_type.on_interact)(&context);

            // Also send event for systems that want to listen
            interaction_events.write(InteractionEvent {
                target_entity,
                source_entity: player_entity,
                interaction_type: interactable.interaction_type.clone(),
            });

            info!("Interaction: {}", interactable.display_name);
        }
    }
}

/// System to manage HoveredInteractable marker component based on cursor position
pub fn update_hovered_interactable(
    mut commands: Commands,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    interactables: Query<(Entity, &Interactable, &Transform, &InteractableHighlight), Without<HoveredInteractable>>,
    current_hovered: Query<Entity, With<HoveredInteractable>>,
) {
    // Get the primary window and main camera
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };

    // Get cursor position and convert to world coordinates
    let world_cursor = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok());

    const HOVER_RADIUS: f32 = 60.0; // Hover detection radius

    // Find the closest interactable under the cursor
    let mut closest_hovered: Option<(Entity, f32)> = None;

    if let Some(cursor_world) = world_cursor {
        for (entity, interactable, transform, highlight) in interactables.iter() {
            if !interactable.is_enabled || !highlight.in_range {
                continue;
            }

            let hover_distance = cursor_world.distance(transform.translation.truncate());
            if hover_distance <= HOVER_RADIUS {
                if let Some((_, closest_distance)) = closest_hovered {
                    if hover_distance < closest_distance {
                        closest_hovered = Some((entity, hover_distance));
                    }
                } else {
                    closest_hovered = Some((entity, hover_distance));
                }
            }
        }
    }

    // Remove HoveredInteractable from current entity if it exists
    for entity in current_hovered.iter() {
        commands.entity(entity).remove::<HoveredInteractable>();
    }

    // Add HoveredInteractable to the new closest entity
    if let Some((entity, _)) = closest_hovered {
        commands.entity(entity).insert(HoveredInteractable);
    }
}

/// Optimized system to detect mouse hover and player range for interactables
pub fn update_interactable_highlights(
    mut interactables: Query<(Entity, &mut InteractableHighlight, &Interactable, &Transform)>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    player_transform: Query<&Transform, (With<Player>, Without<Interactable>)>,
) {
    // Get the primary window and main camera
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };

    // Get player position once for all interactables
    let player_pos = player_transform.single().ok();

    // Get cursor position and convert to world coordinates
    let world_cursor = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok());

    const HOVER_RADIUS: f32 = 60.0; // Hover detection radius

    // Update all interactables in a single pass
    for (_entity, mut highlight, interactable, transform) in interactables.iter_mut() {
        let was_hovered = highlight.is_hovered;
        let was_in_range = highlight.in_range;

        // Update hover state
        highlight.is_hovered = if let Some(cursor_world) = world_cursor {
            let hover_distance = cursor_world.distance(transform.translation.truncate());
            hover_distance <= HOVER_RADIUS && interactable.is_enabled
        } else {
            false
        };

        // Update range state
        highlight.in_range = if let Some(player_pos) = player_pos {
            let player_distance = player_pos.translation.distance(transform.translation);
            player_distance <= interactable.interaction_range
        } else {
            false
        };

        // Log state changes for debugging (only when states actually change)
        if highlight.is_hovered != was_hovered {
            debug!(
                "Hover state changed for '{}': {}",
                interactable.display_name,
                if highlight.is_hovered { "hovered" } else { "unhovered" }
            );
        }

        if highlight.in_range != was_in_range {
            debug!(
                "Range state changed for '{}': {}",
                interactable.display_name,
                if highlight.in_range { "in range" } else { "out of range" }
            );
        }
    }
}

/// Component to mark an entity as a halo effect
#[derive(Component)]
pub struct HaloEffect {
    /// The interactable entity this halo belongs to
    pub parent_entity: Entity,
}

/// Consolidated system to manage all halo effects for interactables
/// Handles spawning, positioning, color updates, and cleanup in one system
pub fn manage_halo_effects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut interactables: Query<(Entity, &mut InteractableHighlight, &Transform), With<Interactable>>,
    mut halos: Query<(&mut Transform, &MeshMaterial2d<ColorMaterial>), (With<HaloEffect>, Without<Interactable>)>,
) {
    for (entity, mut highlight, transform) in interactables.iter_mut() {
        // Handle halo spawning
        if highlight.is_hovered && highlight.halo_entity.is_none() {
            let current_color = highlight.current_color();
            let halo_entity = commands.spawn((
                Mesh2d(meshes.add(Circle::new(80.0 * highlight.halo_radius))),
                MeshMaterial2d(materials.add(ColorMaterial::from(current_color))),
                Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, -0.1)),
                HaloEffect { parent_entity: entity },
            )).id();
            highlight.halo_entity = Some(halo_entity);
        }
        // Handle halo cleanup
        else if !highlight.is_hovered && highlight.halo_entity.is_some() {
            if let Some(halo_entity) = highlight.halo_entity.take() {
                commands.entity(halo_entity).despawn();
            }
        }
        // Handle halo updates (position and color) for existing halos
        else if let Some(halo_entity) = highlight.halo_entity {
            if let Ok((mut halo_transform, material_handle)) = halos.get_mut(halo_entity) {
                // Update position
                halo_transform.translation = transform.translation + Vec3::new(0.0, 0.0, -0.1);

                // Update color if highlight state changed
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    let target_color = highlight.current_color();
                    if material.color != target_color {
                        material.color = target_color;
                    }
                }
            }
        }
    }
}
