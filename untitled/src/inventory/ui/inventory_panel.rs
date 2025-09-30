use bevy::prelude::*;
use crate::inventory::{
    Inventory, InstanceId, GridPosition, ItemRotation,
};
use super::DragState;

/// Component to mark the main inventory panel container
#[derive(Component)]
pub struct InventoryPanel;

/// Component to mark a container inventory panel
#[derive(Component)]
pub struct ContainerPanel {
    pub container_entity: Entity,
}

/// Component to mark individual inventory grid cells
#[derive(Component)]
pub struct InventoryCell {
    pub grid_x: u32,
    pub grid_y: u32,
}

/// Component to mark rendered item icons in the inventory
#[derive(Component)]
pub struct InventoryItemIcon {
    pub instance_id: InstanceId,
}

/// Resource to track inventory panel state
#[derive(Resource, Default)]
pub struct InventoryUiState {
    pub is_open: bool,
    pub selected_item: Option<InstanceId>,
    pub open_container: Option<Entity>, // Currently open container
}

// Placeholder color constants for different cell states
pub const CELL_EMPTY_COLOR: Color = Color::srgb(0.2, 0.2, 0.3);
pub const CELL_OCCUPIED_COLOR: Color = Color::srgb(0.3, 0.3, 0.4);
pub const CELL_HOVER_COLOR: Color = Color::srgb(0.4, 0.4, 0.5);
pub const CELL_SELECTED_COLOR: Color = Color::srgb(0.5, 0.3, 0.3);

// UI layout constants
pub const CELL_SIZE: f32 = 40.0;
pub const CELL_SPACING: f32 = 2.0;
pub const PANEL_PADDING: f32 = 10.0;

/// System to handle opening/closing the inventory panel
pub fn toggle_inventory_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<InventoryUiState>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        ui_state.is_open = !ui_state.is_open;
        info!("Inventory panel {}", if ui_state.is_open { "opened" } else { "closed" });
    }
}

/// System to spawn the inventory panel when it should be visible
pub fn spawn_inventory_panel(
    mut commands: Commands,
    ui_state: Res<InventoryUiState>,
    existing_panels: Query<Entity, With<InventoryPanel>>,
) {
    // If panel should be open but doesn't exist, create it
    if ui_state.is_open && existing_panels.is_empty() {
        spawn_panel(&mut commands);
    }
    // If panel should be closed but exists, remove it
    else if !ui_state.is_open && !existing_panels.is_empty() {
        for entity in existing_panels.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Helper function to create the inventory panel UI
fn spawn_panel(commands: &mut Commands) {
    // Calculate panel size based on standard inventory dimensions (6x4 for player)
    let panel_width = (CELL_SIZE + CELL_SPACING) * 6.0 + PANEL_PADDING * 2.0;
    let panel_height = (CELL_SIZE + CELL_SPACING) * 4.0 + PANEL_PADDING * 2.0;

    // Create the main panel container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(100.0), // Position it away from health bar
                top: Val::Px(100.0),
                width: Val::Px(panel_width),
                height: Val::Px(panel_height),
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                border: UiRect::all(Val::Px(2.0)),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(6, 1.0),
                grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
                row_gap: Val::Px(CELL_SPACING),
                column_gap: Val::Px(CELL_SPACING),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            BorderColor(Color::srgb(0.6, 0.6, 0.6)),
            InventoryPanel,
        ))
        .with_children(|parent| {
            // Create grid cells (6x4 = 24 cells)
            for y in 0..4 {
                for x in 0..6 {
                    parent.spawn((
                        Button,
                        Node {
                            width: Val::Px(CELL_SIZE),
                            height: Val::Px(CELL_SIZE),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(CELL_EMPTY_COLOR),
                        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                        InventoryCell { grid_x: x, grid_y: y },
                    ));
                }
            }
        });
}

/// System to update the visual state of inventory cells based on content
pub fn update_inventory_display(
    mut commands: Commands,
    mut cell_query: Query<(Entity, &mut BackgroundColor, &InventoryCell), With<InventoryCell>>,
    item_icon_query: Query<Entity, With<InventoryItemIcon>>,
    player_query: Query<&Inventory, With<crate::components::Player>>,
    item_registry: Res<crate::inventory::ItemRegistry>,
    ui_state: Res<InventoryUiState>,
) {
    if !ui_state.is_open {
        return;
    }

    // Clean up existing item icons
    for entity in item_icon_query.iter() {
        commands.entity(entity).despawn();
    }

    if let Ok(inventory) = player_query.single() {
        for (cell_entity, mut bg_color, cell) in cell_query.iter_mut() {
            let pos = GridPosition::new(cell.grid_x, cell.grid_y);

            // Check if this cell is occupied
            if let Some(item) = inventory.get_item_at(pos) {
                // Update background color
                if ui_state.selected_item == Some(item.id) {
                    bg_color.0 = CELL_SELECTED_COLOR;
                } else {
                    bg_color.0 = CELL_OCCUPIED_COLOR;
                }

                // Add item icon and text
                if let Some(definition) = item_registry.get(item.item_id) {
                    commands.entity(cell_entity).with_children(|parent| {
                        // Item icon placeholder (using colored rectangle for now)
                        parent.spawn((
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(4.0),
                                top: Val::Px(4.0),
                                ..default()
                            },
                            BackgroundColor(get_item_icon_color(&definition.name)),
                            InventoryItemIcon { instance_id: item.id },
                        ));

                        // Stack size indicator (if > 1)
                        if item.stack_size > 1 {
                            parent.spawn((
                                Text::new(item.stack_size.to_string()),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    position_type: PositionType::Absolute,
                                    right: Val::Px(2.0),
                                    bottom: Val::Px(2.0),
                                    ..default()
                                },
                                InventoryItemIcon { instance_id: item.id },
                            ));
                        }
                    });
                }
            } else {
                bg_color.0 = CELL_EMPTY_COLOR;
            }
        }
    }
}

/// Helper function to get item icon color based on item name (placeholder)
fn get_item_icon_color(item_name: &str) -> Color {
    match item_name {
        "Health Potion" => Color::srgb(1.0, 0.2, 0.2), // Red
        "Rifle" => Color::srgb(0.4, 0.4, 0.4), // Gray
        "Body Armor" => Color::srgb(0.6, 0.4, 0.2), // Brown
        "Ammo" => Color::srgb(1.0, 1.0, 0.2), // Yellow
        _ => Color::srgb(0.5, 0.5, 0.8), // Default blue
    }
}

/// System to handle mouse clicks on inventory cells
pub fn handle_cell_clicks(
    interaction_query: Query<(&Interaction, &InventoryCell), Changed<Interaction>>,
    player_query: Query<&Inventory, With<crate::components::Player>>,
    mut ui_state: ResMut<InventoryUiState>,
) {
    for (interaction, cell) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(inventory) = player_query.single() {
                let pos = GridPosition::new(cell.grid_x, cell.grid_y);

                if let Some(item) = inventory.get_item_at(pos) {
                    // Select/deselect item
                    if ui_state.selected_item == Some(item.id) {
                        ui_state.selected_item = None;

                    } else {
                        ui_state.selected_item = Some(item.id);

                    }
                } else {
                    // Clicked empty cell, deselect any selected item
                    ui_state.selected_item = None;
                }
            }
        }
    }
}

/// System to handle drag and drop interactions
pub fn handle_drag_and_drop(
    mut drag_state: ResMut<DragState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    interaction_query: Query<(&Interaction, &InventoryCell), Changed<Interaction>>,
    mut player_query: Query<&mut Inventory, With<crate::components::Player>>,
    item_registry: Res<crate::inventory::ItemRegistry>,
    ui_state: Res<InventoryUiState>,
) {
    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };

    // Update current mouse position
    drag_state.current_mouse_position = cursor_pos;

    // Start drag on mouse down
    if mouse_input.just_pressed(MouseButton::Left) && !drag_state.is_dragging {
        for (interaction, cell) in interaction_query.iter() {
            if *interaction == Interaction::Pressed {
                if let Ok(inventory) = player_query.single_mut() {
                    let pos = GridPosition::new(cell.grid_x, cell.grid_y);

                    if let Some(item) = inventory.get_item_at(pos) {
                        // Start dragging
                        drag_state.is_dragging = false; // Will become true after threshold
                        drag_state.dragged_item = Some(item.id);
                        drag_state.drag_start_position = cursor_pos;
                        drag_state.original_grid_position = Some(pos);
                        drag_state.current_rotation = item.rotation;
                        drag_state.drag_offset = Vec2::ZERO;
                        break;
                    }
                }
            }
        }
    }

    // Check if we should start actual dragging (past threshold)
    if drag_state.dragged_item.is_some() && !drag_state.is_dragging {
        let drag_distance = (cursor_pos - drag_state.drag_start_position).length();
        if drag_distance > drag_state.drag_threshold {
            drag_state.is_dragging = true;
        }
    }

    // Handle rotation during drag
    if drag_state.is_dragging {
        let should_rotate = mouse_input.just_pressed(MouseButton::Right) ||
                           keyboard_input.just_pressed(KeyCode::KeyR);

        if should_rotate {
            if let Some(item_id) = drag_state.dragged_item {
                if let Ok(inventory) = player_query.single() {
                    if let Some(item) = inventory.grid.items.get(&item_id) {
                        // Check if item can be rotated
                        if let Some(definition) = item_registry.get(item.item_id) {
                            if definition.can_rotate {
                                // Only rotate the drag state rotation, not the actual inventory item
                                drag_state.current_rotation = drag_state.current_rotation.rotate_clockwise();
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle rotation for selected items when not dragging
    if !drag_state.is_dragging {
        let should_rotate = mouse_input.just_pressed(MouseButton::Right) ||
                           keyboard_input.just_pressed(KeyCode::KeyR);

        if should_rotate {
            if let Some(selected_id) = ui_state.selected_item {
                if let Ok(mut inventory) = player_query.single_mut() {
                    if inventory.grid.items.contains_key(&selected_id) {
                        match inventory.rotate_item(selected_id, &item_registry) {
                            Ok(()) => {
                                // Successfully rotated
                            }
                            Err(_) => {
                                // Rotation failed (item can't be rotated or not enough space)
                            }
                        }
                    }
                }
            }
        }
    }

    // Stop drag on mouse release
    if mouse_input.just_released(MouseButton::Left) && drag_state.dragged_item.is_some() {
        if drag_state.is_dragging {


            // Handle drop logic
            handle_item_drop(&drag_state, &interaction_query, &mut player_query, &item_registry);
        }

        // Reset drag state
        *drag_state = DragState::default();
    }
}

/// Handle dropping an item at the current mouse position
fn handle_item_drop(
    drag_state: &DragState,
    _interaction_query: &Query<(&Interaction, &InventoryCell), Changed<Interaction>>,
    player_query: &mut Query<&mut Inventory, With<crate::components::Player>>,
    item_registry: &Res<crate::inventory::ItemRegistry>,
) {
    let Some(item_id) = drag_state.dragged_item else { return; };
    let Ok(mut inventory) = player_query.single_mut() else { return; };

    // Convert mouse position to grid coordinates (same logic as validation)
    let inventory_start_x = 130.0;
    let inventory_start_y = 130.0;
    let cell_size = 40.0;

    let relative_x = drag_state.current_mouse_position.x - inventory_start_x;
    let relative_y = drag_state.current_mouse_position.y - inventory_start_y;

    let mut target_pos = None;

    // Check if mouse is within inventory bounds
    if relative_x >= 0.0 && relative_y >= 0.0 {
        let grid_x = (relative_x / cell_size) as u32;
        let grid_y = (relative_y / cell_size) as u32;

        // Check if grid position is within inventory bounds (10x10 grid)
        if grid_x < 10 && grid_y < 10 {
            target_pos = Some(GridPosition::new(grid_x, grid_y));
        }
    }

    // If mouse is outside inventory, fall back to original position
    if target_pos.is_none() {
        target_pos = drag_state.original_grid_position;
    }

    // If rotation changed, original position might not work anymore
    if let Some(original_item) = inventory.grid.items.get(&item_id) {
        if original_item.rotation != drag_state.current_rotation {
            // Need to check if item still fits at original position with new rotation
            if let Some(definition) = item_registry.get(original_item.item_id) {
                let rotated_size = drag_state.current_rotation.apply_to_size(definition.size);
                if let Some(original_pos) = drag_state.original_grid_position {
                    // Temporarily remove item to check if it fits
                    let temp_item = inventory.remove_item(item_id);
                    let fits = inventory.grid.is_area_free(original_pos, rotated_size);

                    // Put item back temporarily
                    if let Some(item) = temp_item {
                        inventory.grid.items.insert(item_id, item);
                    }

                    if !fits {
                        target_pos = None; // Force auto-placement
                    }
                }
            }
        }
    }

    // Attempt to move the item to the new position
    // First, remove the item from its current position
    if let Some(mut item) = inventory.remove_item(item_id) {
        // Update the rotation
        item.rotation = drag_state.current_rotation;

        let mut placement_succeeded = false;

        // First, try stacking with existing items
        match inventory.try_stack_item(item.clone(), item_registry) {
            Ok(()) => {

                placement_succeeded = true;
            }
            Err((remaining_item, _)) => {

                // Update item with any remaining quantity
                item = remaining_item;
                // Continue to placement logic if there's still item left
            }
        }

        // If stacking didn't fully consume the item, try placement at target position
        if !placement_succeeded && target_pos.is_some() {
            if let Some(target_position) = target_pos {
                match inventory.try_place_item(item.clone(), target_position, item_registry) {
                    Ok(()) => {

                        placement_succeeded = true;
                    }
                    Err(_) => {

                    }
                }
            }
        }

        // If placement at target failed, try auto-placement
        if !placement_succeeded {
            match inventory.auto_place_item(item.clone(), item_registry) {
                Ok(()) => {

                    placement_succeeded = true;
                }
                Err(_) => {

                }
            }
        }

        // If both failed, try to put it back in original position without rotation
        if !placement_succeeded {
            let mut fallback_item = item;
            fallback_item.rotation = ItemRotation::None; // Reset rotation
            if let Some(original_pos) = drag_state.original_grid_position {
                if let Err(_) = inventory.try_place_item(fallback_item.clone(), original_pos, item_registry) {
                    // Last resort: force auto-placement without rotation
                    let _ = inventory.auto_place_item(fallback_item, item_registry);
                }
            } else {
                // No original position, force auto-placement
                let _ = inventory.auto_place_item(fallback_item, item_registry);
            }
        }
    }
}
