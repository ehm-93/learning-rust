use bevy::prelude::*;
use crate::{
    inventory::{InstanceId, GridPosition, ItemRotation},
    player::Player,
};

/// Component for drag preview visual elements
#[derive(Component)]
pub struct DragPreview {
    pub item_id: InstanceId,
}

/// Resource to track drag state
#[derive(Resource)]
pub struct DragState {
    pub is_dragging: bool,
    pub dragged_item: Option<InstanceId>,
    pub drag_start_position: Vec2,
    pub current_mouse_position: Vec2,
    pub drag_offset: Vec2,
    pub original_grid_position: Option<GridPosition>,
    pub current_rotation: ItemRotation,
    pub drag_threshold: f32,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            dragged_item: None,
            drag_start_position: Vec2::ZERO,
            current_mouse_position: Vec2::ZERO,
            drag_offset: Vec2::ZERO,
            original_grid_position: None,
            current_rotation: ItemRotation::None,
            drag_threshold: 5.0, // Pixels before drag starts
        }
    }
}

/// System to handle drag preview updates
pub fn update_drag_preview(
    drag_state: Res<DragState>,
    mut preview_query: Query<(&mut Node, &mut BackgroundColor), With<DragPreview>>,
    cell_query: Query<(&crate::inventory::ui::InventoryCell, &GlobalTransform)>,
    player_query: Query<&crate::inventory::Inventory, With<Player>>,
    item_registry: Res<crate::inventory::ItemRegistry>,
) {
    if !drag_state.is_dragging {
        return;
    }

    // Update drag preview position and color based on validity
    for (mut node, mut bg_color) in preview_query.iter_mut() {
        // Get current item and calculate rotated size
        let Some(item_id) = drag_state.dragged_item else { continue; };
        let Ok(inventory) = player_query.single() else { continue; };
        let Some(item) = inventory.grid.items.get(&item_id) else { continue; };
        let Some(definition) = item_registry.get(item.item_id) else { continue; };

        // Calculate current preview size based on rotation
        let rotated_size = drag_state.current_rotation.apply_to_size(definition.size);
        let cell_size = 40.0;
        let preview_size = Vec2::new(
            rotated_size.width as f32 * cell_size,
            rotated_size.height as f32 * cell_size
        );

        // Update size to match current rotation
        node.width = Val::Px(preview_size.x);
        node.height = Val::Px(preview_size.y);

        // Update position so cursor is at center of top-left cell
        let half_cell = cell_size / 2.0;
        node.left = Val::Px(drag_state.current_mouse_position.x - half_cell);
        node.top = Val::Px(drag_state.current_mouse_position.y - half_cell);

        // Check if current position is a valid drop location
        let is_valid_drop = check_drop_validity(
            &drag_state,
            &cell_query,
            &player_query,
            &item_registry,
        );

        // Update color based on validity (green for valid, red for invalid)
        bg_color.0 = if is_valid_drop {
            Color::srgba(0.2, 1.0, 0.2, 0.6) // Green
        } else {
            Color::srgba(1.0, 0.2, 0.2, 0.6) // Red
        };
    }
}

/// Helper function to check if the current drag position is valid for dropping
fn check_drop_validity(
    drag_state: &DragState,
    _cell_query: &Query<(&crate::inventory::ui::InventoryCell, &GlobalTransform)>,
    player_query: &Query<&crate::inventory::Inventory, With<Player>>,
    item_registry: &Res<crate::inventory::ItemRegistry>,
) -> bool {
    let Some(item_id) = drag_state.dragged_item else { return false; };
    let Ok(inventory) = player_query.single() else { return false; };
    let Some(item) = inventory.grid.items.get(&item_id) else { return false; };
    let Some(definition) = item_registry.get(item.item_id) else { return false; };

    // Convert mouse position to grid coordinates
    // Inventory panel starts at approximately (130, 130) with 40px cells
    let inventory_start_x = 130.0;
    let inventory_start_y = 130.0;
    let cell_size = 40.0;

    let relative_x = drag_state.current_mouse_position.x - inventory_start_x;
    let relative_y = drag_state.current_mouse_position.y - inventory_start_y;

    // Check if mouse is within inventory bounds
    if relative_x < 0.0 || relative_y < 0.0 {
        return false;
    }

    let grid_x = (relative_x / cell_size) as u32;
    let grid_y = (relative_y / cell_size) as u32;

    // Check if grid position is within inventory bounds (10x10 grid)
    if grid_x >= 10 || grid_y >= 10 {
        return false;
    }

    let target_pos = GridPosition::new(grid_x, grid_y);

    // Check if the item can be placed at this position with current rotation
    let rotated_size = drag_state.current_rotation.apply_to_size(definition.size);

    // Temporarily remove the item from inventory to check placement
    let mut temp_inventory = inventory.clone();
    temp_inventory.remove_item(item_id);

    // Check if the area is free
    temp_inventory.grid.is_area_free(target_pos, rotated_size)
}

/// System to spawn drag preview when dragging starts
pub fn spawn_drag_preview(
    mut commands: Commands,
    drag_state: Res<DragState>,
    existing_preview: Query<Entity, With<DragPreview>>,
    player_query: Query<&crate::inventory::Inventory, With<Player>>,
    item_registry: Res<crate::inventory::ItemRegistry>,
) {
    if drag_state.is_dragging && drag_state.dragged_item.is_some() && existing_preview.is_empty() {
        let Some(item_id) = drag_state.dragged_item else { return; };

        // Find the item in player's inventory
        let Ok(inventory) = player_query.single() else { return; };
        let Some(item) = inventory.grid.items.get(&item_id) else { return; };

        // Get item definition for visual properties
        let Some(definition) = item_registry.get(item.item_id) else { return; };

        // Calculate display size based on current rotation
        let rotated_size = drag_state.current_rotation.apply_to_size(definition.size);
        let cell_size = 40.0; // Match inventory panel cell size
        let preview_size = Vec2::new(
            rotated_size.width as f32 * cell_size,
            rotated_size.height as f32 * cell_size
        );

        // Create drag preview visual as UI element (not world sprite)
        // Position cursor at center of top-left cell
        let half_cell = cell_size / 2.0;
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(drag_state.current_mouse_position.x - half_cell),
                top: Val::Px(drag_state.current_mouse_position.y - half_cell),
                width: Val::Px(preview_size.x),
                height: Val::Px(preview_size.y),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.8)), // Bright red for visibility
            DragPreview { item_id },
        ));
    }
}

/// System to cleanup drag preview when dragging ends
pub fn cleanup_drag_preview(
    mut commands: Commands,
    drag_state: Res<DragState>,
    preview_query: Query<Entity, With<DragPreview>>,
) {
    if !drag_state.is_dragging {
        // Clean up any existing drag previews
        for entity in preview_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
