use bevy::prelude::*;
use crate::inventory::{
    Inventory, InstanceId, GridPosition,
};

/// Component to mark the main inventory panel container
#[derive(Component)]
pub struct InventoryPanel;

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
                        info!("Deselected item: {:?}", item.id);
                    } else {
                        ui_state.selected_item = Some(item.id);
                        info!("Selected item: {:?}", item.id);
                    }
                } else {
                    // Clicked empty cell, deselect any selected item
                    ui_state.selected_item = None;
                }
            }
        }
    }
}
