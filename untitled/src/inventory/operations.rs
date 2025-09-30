use bevy::prelude::*;

use crate::inventory::{
    components::{Inventory, ItemInstance, InstanceId, GridPosition, GridCell, ItemRotation},
    registry::{ItemRegistry, GridSize},
};

/// Errors that can occur during inventory operations
#[derive(Debug, Clone)]
pub enum InventoryError {
    /// No space available for the item
    NoSpace,
    /// Invalid position specified
    InvalidPosition,
    /// Area is already occupied
    AreaOccupied,
    /// Item not found in inventory
    ItemNotFound,
    /// Operation not allowed (e.g., rotation on non-rotatable item)
    NotAllowed,
    /// Item cannot be stacked
    CannotStack,
    /// Inventory is at maximum capacity
    AtCapacity,
}

impl std::fmt::Display for InventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InventoryError::NoSpace => write!(f, "No space available in inventory"),
            InventoryError::InvalidPosition => write!(f, "Invalid position specified"),
            InventoryError::AreaOccupied => write!(f, "Area is already occupied"),
            InventoryError::ItemNotFound => write!(f, "Item not found in inventory"),
            InventoryError::NotAllowed => write!(f, "Operation not allowed"),
            InventoryError::CannotStack => write!(f, "Item cannot be stacked"),
            InventoryError::AtCapacity => write!(f, "Inventory is at maximum capacity"),
        }
    }
}

impl std::error::Error for InventoryError {}

impl Inventory {
    /// Try to add an item at a specific position
    pub fn try_place_item(
        &mut self,
        mut item: ItemInstance,
        position: GridPosition,
        registry: &ItemRegistry,
    ) -> Result<(), InventoryError> {
        let definition = registry
            .get(item.item_id)
            .ok_or(InventoryError::NotAllowed)?;

        // Calculate actual size considering rotation
        let actual_size = item.rotation.apply_to_size(definition.size);

        // Check if position is valid and area is free
        if !self.grid.is_valid_position(position) {
            return Err(InventoryError::InvalidPosition);
        }

        if !self.grid.is_area_free(position, actual_size) {
            return Err(InventoryError::AreaOccupied);
        }

        // Set item position
        item.position = position;

        // Occupy grid cells
        for y in position.y..(position.y + actual_size.height) {
            for x in position.x..(position.x + actual_size.width) {
                let is_origin = x == position.x && y == position.y;
                self.grid.cells[y as usize][x as usize] = Some(GridCell {
                    item_id: item.id,
                    is_origin,
                });
            }
        }

        // Add item to inventory
        self.grid.items.insert(item.id, item);

        Ok(())
    }

    /// Find the best position for an item (auto-placement)
    pub fn find_best_position(
        &self,
        item_size: GridSize,
        allow_rotation: bool,
        _registry: &ItemRegistry,
    ) -> Option<(GridPosition, ItemRotation)> {
        let rotations = if allow_rotation {
            vec![
                ItemRotation::None,
                ItemRotation::Clockwise90,
                ItemRotation::Clockwise180,
                ItemRotation::Clockwise270,
            ]
        } else {
            vec![ItemRotation::None]
        };

        // Try each rotation
        for rotation in rotations {
            let actual_size = rotation.apply_to_size(item_size);

            // Try each position in the grid
            for y in 0..self.grid.config.current_height {
                for x in 0..self.grid.config.current_width {
                    let pos = GridPosition::new(x, y);
                    if self.grid.is_area_free(pos, actual_size) {
                        return Some((pos, rotation));
                    }
                }
            }
        }

        None
    }

    /// Auto-place an item in the first available position
    pub fn auto_place_item(
        &mut self,
        mut item: ItemInstance,
        registry: &ItemRegistry,
    ) -> Result<(), InventoryError> {
        let definition = registry
            .get(item.item_id)
            .ok_or(InventoryError::NotAllowed)?;

        let (position, rotation) = self
            .find_best_position(definition.size, definition.can_rotate && self.grid.config.allow_rotation, registry)
            .ok_or(InventoryError::NoSpace)?;

        item.rotation = rotation;
        self.try_place_item(item, position, registry)
    }

    /// Remove an item from the inventory
    pub fn remove_item(&mut self, instance_id: InstanceId) -> Option<ItemInstance> {
        let item = self.grid.items.remove(&instance_id)?;

        // Clear grid cells occupied by this item
        for y in 0..self.grid.config.current_height {
            for x in 0..self.grid.config.current_width {
                if let Some(cell) = &self.grid.cells[y as usize][x as usize] {
                    if cell.item_id == instance_id {
                        self.grid.cells[y as usize][x as usize] = None;
                    }
                }
            }
        }

        Some(item)
    }

    /// Try to stack an item with existing items
    pub fn try_stack_item(
        &mut self,
        item: ItemInstance,
        registry: &ItemRegistry,
    ) -> Result<(), (ItemInstance, InventoryError)> {
        let definition = registry
            .get(item.item_id)
            .ok_or_else(|| (item.clone(), InventoryError::NotAllowed))?;

        let max_stack = match definition.max_stack_size {
            Some(size) => size,
            None => return Err((item, InventoryError::CannotStack)),
        };

        // Find existing stacks of the same item type
        for existing_item in self.grid.items.values_mut() {
            if existing_item.item_id == item.item_id
                && existing_item.stack_size < max_stack
            {
                let space_available = max_stack - existing_item.stack_size;
                let amount_to_add = item.stack_size.min(space_available);

                existing_item.stack_size += amount_to_add;

                // If we used all of the new item, we're done
                if amount_to_add >= item.stack_size {
                    return Ok(());
                }

                // Otherwise, create a new item with the remaining amount
                let mut remaining_item = item.clone();
                remaining_item.stack_size -= amount_to_add;
                return self.auto_place_item(remaining_item, registry)
                    .map_err(|e| (item, e));
            }
        }

        // No existing stack found, try to place as new item
        let item_clone = item.clone();
        self.auto_place_item(item, registry)
            .map_err(|e| (item_clone, e))
    }

    /// Rotate an item (if possible)
    pub fn rotate_item(
        &mut self,
        instance_id: InstanceId,
        registry: &ItemRegistry,
    ) -> Result<(), InventoryError> {
        let item = self.grid.items.get(&instance_id)
            .ok_or(InventoryError::ItemNotFound)?;

        let definition = registry
            .get(item.item_id)
            .ok_or(InventoryError::NotAllowed)?;

        if !definition.can_rotate || !self.grid.config.allow_rotation {
            return Err(InventoryError::NotAllowed);
        }

        let current_pos = item.position;
        let current_rotation = item.rotation;
        let new_rotation = current_rotation.rotate_clockwise();
        let new_size = new_rotation.apply_to_size(definition.size);

        // Temporarily remove the item
        let mut item = self.remove_item(instance_id).unwrap();

        // Check if new rotation fits
        if self.grid.is_area_free(current_pos, new_size) {
            item.rotation = new_rotation;
            self.try_place_item(item, current_pos, registry)?;
        } else {
            // Rotation doesn't fit, put the item back
            self.try_place_item(item, current_pos, registry)?;
            return Err(InventoryError::NoSpace);
        }

        Ok(())
    }

    /// Get item at a specific grid position
    pub fn get_item_at(&self, position: GridPosition) -> Option<&ItemInstance> {
        self.grid.get_item_at(position)
    }

    /// Get all items in the inventory
    pub fn get_all_items(&self) -> Vec<&ItemInstance> {
        self.grid.items.values().collect()
    }

    /// Get current item count
    pub fn item_count(&self) -> usize {
        self.grid.items.len()
    }

    /// Check if inventory is empty
    pub fn is_empty(&self) -> bool {
        self.grid.items.is_empty()
    }

    /// Get total stack count (counting stacks)
    pub fn total_stack_count(&self) -> u32 {
        self.grid.items.values().map(|item| item.stack_size).sum()
    }

    /// Resize the inventory
    pub fn resize(&mut self, new_width: u32, new_height: u32) -> Result<(), InventoryError> {
        self.grid.resize(new_width, new_height)
            .map_err(|_| InventoryError::InvalidPosition)
    }

    /// Compact inventory by moving items to fill gaps (simple version)
    pub fn compact(&mut self, registry: &ItemRegistry) {
        let items: Vec<ItemInstance> = self.grid.items.values().cloned().collect();

        // Clear the inventory
        self.grid.items.clear();
        for row in &mut self.grid.cells {
            for cell in row {
                *cell = None;
            }
        }

        // Re-add all items using auto-placement
        for item in items {
            let _ = self.auto_place_item(item, registry);
        }
    }
}

/// System to handle inventory operations
pub fn inventory_operations_system(
    // This system would handle queued inventory operations in a real implementation
    // For now, it's just a placeholder
) {
    // Operations would be processed here
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{
        registry::{ItemDefinition, ItemId},
        factory::ItemFactory,
    };

    #[test]
    fn test_item_placement() {
        let mut inventory = Inventory::new(4, 4);
        let mut registry = ItemRegistry::new();
        let mut factory = ItemFactory::new();

        // Register a test item
        registry.register(
            ItemDefinition::new(ItemId(1), "Test Item")
                .with_size(2, 1)
        );

        let item = factory.create_item(ItemId(1), &registry).unwrap();

        // Should be able to place at origin
        assert!(inventory.try_place_item(item.clone(), GridPosition::new(0, 0), &registry).is_ok());

        // Should not be able to place overlapping item
        let item2 = factory.create_item(ItemId(1), &registry).unwrap();
        assert!(matches!(
            inventory.try_place_item(item2, GridPosition::new(1, 0), &registry),
            Err(InventoryError::AreaOccupied)
        ));
    }

    #[test]
    fn test_auto_placement() {
        let mut inventory = Inventory::new(3, 3);
        let mut registry = ItemRegistry::new();
        let mut factory = ItemFactory::new();

        registry.register(
            ItemDefinition::new(ItemId(1), "Small Item")
                .with_size(1, 1)
        );

        // Should be able to auto-place multiple items
        for _ in 0..9 {
            let item = factory.create_item(ItemId(1), &registry).unwrap();
            assert!(inventory.auto_place_item(item, &registry).is_ok());
        }

        // 10th item should fail (no space)
        let item = factory.create_item(ItemId(1), &registry).unwrap();
        assert!(matches!(
            inventory.auto_place_item(item, &registry),
            Err(InventoryError::NoSpace)
        ));
    }
}
