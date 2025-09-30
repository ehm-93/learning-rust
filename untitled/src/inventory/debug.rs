use bevy::prelude::*;
use crate::inventory::*;
use crate::components::Player;

/// Debug commands for testing inventory system
pub fn inventory_debug_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut item_factory: ResMut<ItemFactory>,
    item_registry: Res<ItemRegistry>,
    mut player_query: Query<(Entity, Option<&mut Inventory>), With<Player>>,
) {
    if let Ok((player_entity, inventory_opt)) = player_query.single_mut() {

        // I key: Give player an inventory if they don't have one
        if keyboard.just_pressed(KeyCode::KeyI) {
            if inventory_opt.is_none() {
                let inventory = Inventory::player_inventory();
                commands.entity(player_entity).insert(inventory);
                info!("Added inventory to player");
            } else {
                info!("Player already has an inventory");
            }
        }

        if let Some(mut inventory) = inventory_opt {
            // 1 key: Add a health potion
            if keyboard.just_pressed(KeyCode::Digit1) {
                if let Some(item) = item_factory.create_item(ItemId(1), &item_registry) {
                    match inventory.auto_place_item(item, &item_registry) {
                        Ok(()) => info!("Added health potion to inventory"),
                        Err(e) => warn!("Failed to add health potion: {}", e),
                    }
                }
            }

            // 2 key: Add a rifle
            if keyboard.just_pressed(KeyCode::Digit2) {
                if let Some(item) = item_factory.create_item(ItemId(2), &item_registry) {
                    match inventory.auto_place_item(item, &item_registry) {
                        Ok(()) => info!("Added rifle to inventory"),
                        Err(e) => warn!("Failed to add rifle: {}", e),
                    }
                }
            }

            // 3 key: Add armor
            if keyboard.just_pressed(KeyCode::Digit3) {
                if let Some(item) = item_factory.create_item(ItemId(3), &item_registry) {
                    match inventory.auto_place_item(item, &item_registry) {
                        Ok(()) => info!("Added armor to inventory"),
                        Err(e) => warn!("Failed to add armor: {}", e),
                    }
                }
            }

            // C key: Clear inventory
            if keyboard.just_pressed(KeyCode::KeyC) {
                let item_count = inventory.item_count();
                inventory.grid.items.clear();
                for row in &mut inventory.grid.cells {
                    for cell in row {
                        *cell = None;
                    }
                }
                info!("Cleared inventory ({} items removed)", item_count);
            }

            // L key: List inventory contents
            if keyboard.just_pressed(KeyCode::KeyL) {
                info!("=== INVENTORY CONTENTS ===");
                info!("Size: {}x{}", inventory.grid.config.current_width, inventory.grid.config.current_height);
                info!("Items: {}", inventory.item_count());

                for item in inventory.get_all_items() {
                    if let Some(definition) = item_registry.get(item.item_id) {
                        info!(
                            "- {} (ID: {:?}) at ({}, {}) rotation: {:?} stack: {}",
                            definition.name,
                            item.id,
                            item.position.x,
                            item.position.y,
                            item.rotation,
                            item.stack_size
                        );

                        // Show properties
                        for (prop_name, prop_value) in &item.properties {
                            info!("  {}: {:.1}", prop_name, prop_value);
                        }
                    }
                }
                info!("========================");
            }

            // R key: Try to rotate first item
            if keyboard.just_pressed(KeyCode::KeyR) {
                let first_item_id = inventory.get_all_items().first().map(|item| item.id);
                if let Some(item_id) = first_item_id {
                    match inventory.rotate_item(item_id, &item_registry) {
                        Ok(()) => info!("Rotated item: {:?}", item_id),
                        Err(e) => warn!("Failed to rotate item: {}", e),
                    }
                }
            }

            // + key: Expand inventory
            if keyboard.just_pressed(KeyCode::Equal) {
                let new_width = (inventory.grid.config.current_width + 1).min(10);
                let new_height = inventory.grid.config.current_height;
                match inventory.resize(new_width, new_height) {
                    Ok(()) => info!("Expanded inventory to {}x{}", new_width, new_height),
                    Err(e) => warn!("Failed to expand inventory: {}", e),
                }
            }

            // - key: Shrink inventory
            if keyboard.just_pressed(KeyCode::Minus) {
                let new_width = (inventory.grid.config.current_width.saturating_sub(1)).max(1);
                let new_height = inventory.grid.config.current_height;
                match inventory.resize(new_width, new_height) {
                    Ok(()) => info!("Shrunk inventory to {}x{}", new_width, new_height),
                    Err(e) => warn!("Failed to shrink inventory: {}", e),
                }
            }
        }
    }
}

/// Help system that shows debug commands
pub fn inventory_debug_help_system(
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        info!("=== INVENTORY DEBUG HELP ===");
        info!("I - Give player an inventory (if they don't have one)");
        info!("1 - Add a health potion");
        info!("2 - Add a rifle");
        info!("3 - Add body armor");
        info!("4 - Add ammo (stackable)");
        info!("C - Clear inventory (remove all items)");
        info!("L - List inventory contents");
        info!("R - Rotate first item in inventory");
        info!("+ (Equal) - Expand inventory width");
        info!("- (Minus) - Shrink inventory width");
        info!("");
        info!("=== INVENTORY UI CONTROLS ===");
        info!("Tab - Open/Close inventory panel");
        info!("Click - Select/deselect items in inventory");
        info!("Hover - View item tooltips");
        info!("");
        info!("F1 - Show this help");
        info!("=============================");
    }
}
