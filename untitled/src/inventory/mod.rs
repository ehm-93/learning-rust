pub mod components;
pub mod factory;
pub mod operations;
pub mod registry;
pub mod debug;

// Re-export commonly used types
pub use components::{Inventory, ItemInstance, InstanceId, GridPosition, ItemRotation, InventorySize};
pub use factory::{ItemFactory, ItemCreationError};
pub use operations::{InventoryError};
pub use registry::{ItemRegistry, ItemDefinition, ItemId, PropertyRange, GridSize};

use bevy::prelude::*;

/// Inventory plugin that sets up all inventory systems
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add startup systems
            .add_systems(Startup, (
                registry::setup_item_registry,
                factory::setup_item_factory,
            ))
            // Add update systems
            .add_systems(Update, (
                operations::inventory_operations_system,
                debug::inventory_debug_system,
                debug::inventory_debug_help_system,
            ));
    }
}
