use bevy::prelude::*;
use crate::{
    inventory::{ItemRegistry, InstanceId, ItemInstance, ItemDefinition},
    player::Player,
};

/// Component to mark tooltip UI elements
#[derive(Component)]
pub struct ItemTooltip {
    pub item_id: InstanceId,
}

/// Resource to track tooltip state
#[derive(Resource, Default)]
pub struct TooltipState {
    pub current_item: Option<InstanceId>,
    pub mouse_position: Vec2,
}

/// System to update tooltip state based on mouse interaction with cells
pub fn update_tooltip_state(
    mut tooltip_state: ResMut<TooltipState>,
    windows: Query<&Window>,
    interaction_query: Query<(&Interaction, &crate::inventory::ui::InventoryCell)>,
    player_query: Query<&crate::inventory::Inventory, With<Player>>,
) {
    if let Ok(window) = windows.single() {
        if let Some(cursor_position) = window.cursor_position() {
            tooltip_state.mouse_position = cursor_position;
        }
    }

    // Reset current item first
    tooltip_state.current_item = None;

    // Check for currently hovered cells (without Changed filter)
    if let Ok(inventory) = player_query.single() {
        for (interaction, cell) in interaction_query.iter() {
            if *interaction == Interaction::Hovered {
                let pos = crate::inventory::GridPosition::new(cell.grid_x, cell.grid_y);
                if let Some(item) = inventory.get_item_at(pos) {
                    tooltip_state.current_item = Some(item.id);
                    break;
                }
            }
        }
    }
}

/// System to spawn tooltips for hovered items
pub fn spawn_tooltips(
    mut commands: Commands,
    tooltip_state: Res<TooltipState>,
    existing_tooltips: Query<(Entity, &ItemTooltip)>,
    player_query: Query<&crate::inventory::Inventory, With<Player>>,
    item_registry: Res<ItemRegistry>,
) {
    match tooltip_state.current_item {
        Some(current_item_id) => {
            // Check if we already have a tooltip for this item
            let existing_tooltip = existing_tooltips.iter()
                .find(|(_, tooltip)| tooltip.item_id == current_item_id);

            if existing_tooltip.is_none() {
                // Remove any existing tooltips for different items
                for (entity, _) in existing_tooltips.iter() {
                    commands.entity(entity).despawn();
                }

                // Spawn new tooltip
                if let Ok(inventory) = player_query.single() {
                    if let Some(item) = inventory.get_all_items().iter().find(|item| item.id == current_item_id) {
                        if let Some(definition) = item_registry.get(item.item_id) {
                            spawn_tooltip_for_item(&mut commands, item, definition, tooltip_state.mouse_position);
                        }
                    }
                }
            }
        }
        None => {
            // Remove all tooltips when not hovering
            for (entity, _) in existing_tooltips.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Helper function to create a tooltip UI element
fn spawn_tooltip_for_item(
    commands: &mut Commands,
    item: &ItemInstance,
    definition: &ItemDefinition,
    mouse_pos: Vec2,
) {
    // Create tooltip container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(mouse_pos.x + 10.0), // Offset from cursor
                top: Val::Px(mouse_pos.y - 50.0),  // Above cursor
                width: Val::Px(200.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            BorderColor(Color::srgb(0.6, 0.6, 0.6)),
            ItemTooltip { item_id: item.id },
        ))
        .with_children(|parent| {
            // Item name
            parent.spawn((
                Text::new(&definition.name),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Item description (if available)
            if !definition.description.is_empty() {
                parent.spawn((
                    Text::new(&definition.description),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
            }

            // Stack size (if stackable and > 1)
            if item.stack_size > 1 {
                parent.spawn((
                    Text::new(format!("Quantity: {}", item.stack_size)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.9)),
                    Node {
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..default()
                    },
                ));
            }

            // Item properties
            for (prop_name, prop_value) in &item.properties {
                parent.spawn((
                    Text::new(format!("{}: {:.1}", prop_name, prop_value)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.9, 0.7)),
                    Node {
                        margin: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                ));
            }
        });
}
