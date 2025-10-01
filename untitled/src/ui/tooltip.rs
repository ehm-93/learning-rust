use bevy::prelude::*;

/// Tooltip component that holds content to display
#[derive(Component, Clone)]
pub struct Tooltip {
    /// Text content to display in the tooltip
    pub content: String,
    /// Offset from the entity position where tooltip should appear
    pub offset: Vec3,
    /// Font size for the tooltip text
    pub font_size: f32,
    /// Background color of the tooltip
    pub background_color: Color,
    /// Text color of the tooltip
    pub text_color: Color,
    /// Maximum width before text wrapping
    pub max_width: f32,
    /// Padding around the text
    pub padding: f32,
}

impl Default for Tooltip {
    fn default() -> Self {
        Self {
            content: String::new(),
            offset: Vec3::new(0.0, 50.0, 10.0), // Default above the entity
            font_size: 14.0,
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8), // Semi-transparent black
            text_color: Color::WHITE,
            max_width: 200.0,
            padding: 10.0,
        }
    }
}

impl Tooltip {
    /// Create a new tooltip with content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..default()
        }
    }

    /// Set the tooltip content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Set the tooltip offset from the entity
    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    /// Set the font size
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Set the background color
    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Set the text color
    pub fn with_text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set the maximum width before wrapping
    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    /// Set the padding around the text
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

/// Component to mark tooltip UI entities for cleanup
#[derive(Component)]
pub struct TooltipUI {
    /// The entity that owns this tooltip
    pub owner: Entity,
}

/// Resource to track the currently displayed tooltip
#[derive(Resource, Default)]
pub struct TooltipState {
    /// Currently displayed tooltip entity (if any)
    pub current_tooltip: Option<Entity>,
    /// The owner of the current tooltip
    pub current_owner: Option<Entity>,
}

/// System to handle showing tooltips when hovering over entities with tooltip components
pub fn handle_tooltip_hover(
    mut commands: Commands,
    mut tooltip_state: ResMut<TooltipState>,
    tooltip_query: Query<(Entity, &Tooltip, &Transform), With<Tooltip>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Get cursor position and convert to world coordinates (matching interaction system)
    let world_cursor = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok());

    const HOVER_RADIUS: f32 = 60.0; // Match interaction system hover radius

    // Find the closest tooltip entity under the cursor (matching interaction system logic)
    let mut closest_hovered: Option<(Entity, &Tooltip, &Transform, f32)> = None;

    if let Some(cursor_world) = world_cursor {
        for (entity, tooltip, transform) in tooltip_query.iter() {
            let hover_distance = cursor_world.distance(transform.translation.truncate());
            if hover_distance <= HOVER_RADIUS {
                if let Some((_, _, _, closest_distance)) = closest_hovered {
                    if hover_distance < closest_distance {
                        closest_hovered = Some((entity, tooltip, transform, hover_distance));
                    }
                } else {
                    closest_hovered = Some((entity, tooltip, transform, hover_distance));
                }
            }
        }
    }

    // Handle tooltip display based on closest hovered entity (matching interaction system logic)
    match closest_hovered {
        Some((entity, tooltip, transform, _)) => {
            // Show tooltip if not already showing for this entity
            if tooltip_state.current_owner != Some(entity) {
                // Clean up existing tooltip
                if let Some(current_tooltip) = tooltip_state.current_tooltip {
                    commands.entity(current_tooltip).despawn();
                }

                // Create new tooltip
                let tooltip_entity = spawn_tooltip_ui(&mut commands, tooltip, transform, &mut meshes, &mut materials);
                tooltip_state.current_tooltip = Some(tooltip_entity);
                tooltip_state.current_owner = Some(entity);
            }
        }
        None => {
            // Hide tooltip if cursor is not over any tooltip entity
            if let Some(current_tooltip) = tooltip_state.current_tooltip.take() {
                commands.entity(current_tooltip).despawn();
                tooltip_state.current_owner = None;
            }
        }
    }
}

/// System to handle showing tooltips when close to interactable entities
pub fn handle_tooltip_proximity(
    mut commands: Commands,
    mut tooltip_state: ResMut<TooltipState>,
    tooltip_query: Query<(Entity, &Tooltip, &Transform), With<Tooltip>>,
    player_query: Query<&Transform, (With<crate::player::Player>, Without<Tooltip>)>,
    ui_tooltip_query: Query<Entity, With<TooltipUI>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let mut closest_tooltip: Option<(Entity, &Tooltip, &Transform, f32)> = None;

    // Find the closest tooltip entity within range
    for (entity, tooltip, transform) in tooltip_query.iter() {
        let distance = player_transform.translation.distance(transform.translation);

        // Check if within interaction range (you can adjust this threshold)
        if distance < 100.0 {
            match closest_tooltip {
                Some((_, _, _, closest_distance)) if distance < closest_distance => {
                    closest_tooltip = Some((entity, tooltip, transform, distance));
                }
                None => {
                    closest_tooltip = Some((entity, tooltip, transform, distance));
                }
                _ => {}
            }
        }
    }

    match closest_tooltip {
        Some((entity, tooltip, transform, _)) => {
            // Show tooltip if not already showing for this entity
            if tooltip_state.current_owner != Some(entity) {
                // Clean up existing tooltip
                if let Some(current_tooltip) = tooltip_state.current_tooltip {
                    if let Ok(ui_entity) = ui_tooltip_query.get(current_tooltip) {
                        commands.entity(ui_entity).despawn();
                    }
                }

                // Create new tooltip
                let tooltip_entity = spawn_tooltip_ui(&mut commands, tooltip, transform, &mut meshes, &mut materials);
                tooltip_state.current_tooltip = Some(tooltip_entity);
                tooltip_state.current_owner = Some(entity);
            }
        }
        None => {
            // Hide tooltip if player is not close to any tooltip entity
            if let Some(current_tooltip) = tooltip_state.current_tooltip.take() {
                if let Ok(ui_entity) = ui_tooltip_query.get(current_tooltip) {
                    commands.entity(ui_entity).despawn();
                }
                tooltip_state.current_owner = None;
            }
        }
    }
}

/// Helper function to spawn tooltip UI using 2D text rendering
fn spawn_tooltip_ui(
    commands: &mut Commands,
    tooltip: &Tooltip,
    entity_transform: &Transform,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> Entity {
    let tooltip_position = entity_transform.translation + tooltip.offset;

    // Calculate background size based on text content
    let text_width = tooltip.content.len() as f32 * tooltip.font_size * 0.6; // Rough estimate
    let text_height = tooltip.content.lines().count() as f32 * tooltip.font_size * 1.2;
    let bg_width = (text_width + tooltip.padding * 2.0).min(tooltip.max_width + tooltip.padding * 2.0);
    let bg_height = text_height + tooltip.padding * 2.0;

    // Create background mesh and material
    let background_mesh = meshes.add(Rectangle::new(bg_width, bg_height));
    let background_material = materials.add(tooltip.background_color);

    // Create a parent entity that contains both background and text
    commands.spawn((
        TooltipUI { owner: Entity::PLACEHOLDER },
        Transform::from_translation(tooltip_position),
        Visibility::Inherited,
    )).with_children(|parent| {
        // Background rectangle
        parent.spawn((
            Mesh2d(background_mesh),
            MeshMaterial2d(background_material),
            Transform::from_translation(Vec3::ZERO),
        ));

        // Text
        parent.spawn((
            Text2d::new(&tooltip.content),
            TextFont {
                font_size: tooltip.font_size,
                ..default()
            },
            TextColor(tooltip.text_color),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)), // Slightly above background
        ));
    }).id()
}

/// Cleanup system to remove tooltips when their owner entities are despawned
pub fn cleanup_orphaned_tooltips(
    mut commands: Commands,
    mut tooltip_state: ResMut<TooltipState>,
    tooltip_query: Query<Entity, With<Tooltip>>,
) {
    if let Some(current_owner) = tooltip_state.current_owner {
        // Check if the owner entity still exists
        if tooltip_query.get(current_owner).is_err() {
            // Owner is gone, clean up the tooltip
            if let Some(current_tooltip) = tooltip_state.current_tooltip.take() {
                commands.entity(current_tooltip).despawn();
                tooltip_state.current_owner = None;
            }
        }
    }
}
