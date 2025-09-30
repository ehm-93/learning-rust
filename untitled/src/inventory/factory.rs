use bevy::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use rand::prelude::IndexedRandom;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::inventory::{
    registry::{ItemRegistry, ItemId},
    components::{ItemInstance, InstanceId},
};

/// Global counter for generating unique instance IDs
static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Factory for creating item instances from definitions
#[derive(Resource)]
pub struct ItemFactory {
    /// Random number generator for property rolling
    rng: StdRng,
}

impl Default for ItemFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemFactory {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_rng(&mut rand::rng()),
        }
    }

    /// Create a new item instance from a definition ID
    pub fn create_item(
        &mut self,
        item_id: ItemId,
        registry: &ItemRegistry,
    ) -> Option<ItemInstance> {
        let definition = registry.get(item_id)?;

        // Generate unique instance ID
        let instance_id = InstanceId(INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst));

        let mut instance = ItemInstance::new(instance_id, item_id);

        // Roll all numeric properties
        for (prop_name, prop_range) in &definition.properties.numeric {
            let rolled_value = prop_range.roll(&mut self.rng);
            instance.properties.insert(prop_name.clone(), rolled_value);
        }

        // Copy boolean flags
        for (flag_name, flag_value) in &definition.properties.flags {
            instance.flags.insert(flag_name.clone(), *flag_value);
        }

        // Copy string properties
        for (string_name, string_value) in &definition.properties.strings {
            instance.strings.insert(string_name.clone(), string_value.clone());
        }

        // Set durability if the item has it
        if let Some(durability) = instance.properties.get("durability") {
            instance.durability = Some(*durability);
        }

        Some(instance)
    }

    /// Create multiple items at once
    pub fn create_items(
        &mut self,
        item_id: ItemId,
        count: u32,
        registry: &ItemRegistry,
    ) -> Vec<ItemInstance> {
        let mut items = Vec::with_capacity(count as usize);
        for _ in 0..count {
            if let Some(item) = self.create_item(item_id, registry) {
                items.push(item);
            }
        }
        items
    }

    /// Create an item with specific property overrides
    pub fn create_item_with_overrides(
        &mut self,
        item_id: ItemId,
        registry: &ItemRegistry,
        property_overrides: &[(String, f32)],
    ) -> Option<ItemInstance> {
        let mut instance = self.create_item(item_id, registry)?;

        // Apply overrides
        for (prop_name, prop_value) in property_overrides {
            instance.properties.insert(prop_name.clone(), *prop_value);
        }

        Some(instance)
    }

    /// Create an item with a specific seed for reproducible results
    pub fn create_item_with_seed(
        &mut self,
        item_id: ItemId,
        registry: &ItemRegistry,
        seed: u64,
    ) -> Option<ItemInstance> {
        // Temporarily use a seeded RNG
        let mut seeded_rng = StdRng::seed_from_u64(seed);
        let definition = registry.get(item_id)?;

        let instance_id = InstanceId(INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst));
        let mut instance = ItemInstance::new(instance_id, item_id);

        // Roll properties with seeded RNG
        for (prop_name, prop_range) in &definition.properties.numeric {
            let rolled_value = prop_range.roll(&mut seeded_rng);
            instance.properties.insert(prop_name.clone(), rolled_value);
        }

        // Copy other properties
        for (flag_name, flag_value) in &definition.properties.flags {
            instance.flags.insert(flag_name.clone(), *flag_value);
        }

        for (string_name, string_value) in &definition.properties.strings {
            instance.strings.insert(string_name.clone(), string_value.clone());
        }

        if let Some(durability) = instance.properties.get("durability") {
            instance.durability = Some(*durability);
        }

        Some(instance)
    }

    /// Generate a random item from a category
    pub fn create_random_from_category(
        &mut self,
        category: &str,
        registry: &ItemRegistry,
    ) -> Option<ItemInstance> {
        let items_in_category = registry.get_category(category);
        if items_in_category.is_empty() {
            return None;
        }

        let chosen_def = items_in_category.choose(&mut self.rng)?;
        self.create_item(chosen_def.id, registry)
    }
}

/// Errors that can occur during item creation
#[derive(Debug, Clone)]
pub enum ItemCreationError {
    DefinitionNotFound(ItemId),
    InvalidPropertyRange,
    FactoryError(String),
}

impl std::fmt::Display for ItemCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemCreationError::DefinitionNotFound(id) => {
                write!(f, "Item definition not found: {:?}", id)
            }
            ItemCreationError::InvalidPropertyRange => {
                write!(f, "Invalid property range specified")
            }
            ItemCreationError::FactoryError(msg) => {
                write!(f, "Item factory error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ItemCreationError {}

/// System to initialize the item factory
pub fn setup_item_factory(mut commands: Commands) {
    commands.insert_resource(ItemFactory::new());
}

/// Helper function to create a stack of items if the definition allows it
pub fn create_stack(
    factory: &mut ItemFactory,
    item_id: ItemId,
    stack_size: u32,
    registry: &ItemRegistry,
) -> Option<ItemInstance> {
    let definition = registry.get(item_id)?;

    // Check if item is stackable
    let max_stack = definition.max_stack_size?;
    if max_stack == 0 {
        return None;
    }

    let mut item = factory.create_item(item_id, registry)?;
    item.stack_size = stack_size.min(max_stack);

    Some(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::registry::{ItemDefinition, PropertyRange};

    #[test]
    fn test_item_creation() {
        let mut factory = ItemFactory::new();
        let mut registry = ItemRegistry::new();

        // Register a test item
        let test_item = ItemDefinition::new(ItemId(999), "Test Item")
            .with_property("damage", PropertyRange::Range { min: 10.0, max: 20.0 })
            .with_property("speed", PropertyRange::Fixed(5.0));

        registry.register(test_item);

        // Create an instance
        let instance = factory.create_item(ItemId(999), &registry).unwrap();

        assert_eq!(instance.item_id, ItemId(999));
        assert!(instance.properties.contains_key("damage"));
        assert!(instance.properties.contains_key("speed"));

        let damage = instance.get_property("damage").unwrap();
        assert!(damage >= 10.0 && damage <= 20.0);

        let speed = instance.get_property("speed").unwrap();
        assert_eq!(speed, 5.0);
    }

    #[test]
    fn test_seeded_creation() {
        let mut factory = ItemFactory::new();
        let mut registry = ItemRegistry::new();

        let test_item = ItemDefinition::new(ItemId(998), "Seeded Item")
            .with_property("random_stat", PropertyRange::Range { min: 1.0, max: 100.0 });

        registry.register(test_item);

        // Create two items with the same seed
        let item1 = factory.create_item_with_seed(ItemId(998), &registry, 12345).unwrap();
        let item2 = factory.create_item_with_seed(ItemId(998), &registry, 12345).unwrap();

        // They should have the same random properties (but different instance IDs)
        assert_ne!(item1.id, item2.id);
        assert_eq!(
            item1.get_property("random_stat"),
            item2.get_property("random_stat")
        );
    }
}
