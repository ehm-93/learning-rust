pub mod components;
pub mod factory;
pub mod operations;
pub mod registry;
pub mod events;
pub mod ui;

// Re-export commonly used types
pub use components::{Inventory, ItemInstance, InstanceId, GridPosition, ItemRotation};
pub use registry::{ItemRegistry, ItemDefinition};
pub use events::*;

use bevy::prelude::*;

/// Inventory plugin that sets up all inventory systems
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add events
            .add_event::<InventoryEvent>()
            .add_event::<events::TooltipEvent>()
            // Add resources
            .init_resource::<ui::InventoryUiState>()
            .init_resource::<ui::TooltipState>()
            .init_resource::<ui::DragState>()
            // Add startup systems
            .add_systems(Startup, (
                registry::setup_item_registry,
                factory::setup_item_factory,
            ))
            // Add update systems
            .add_systems(Update, (
                // Core inventory systems
                operations::inventory_operations_system,
                // UI systems
                ui::toggle_inventory_panel,
                ui::spawn_inventory_panel,
                ui::update_inventory_display,
                ui::handle_cell_clicks,
                ui::handle_drag_and_drop,
                ui::update_tooltip_state,
                ui::spawn_tooltips,
                // Drag and drop systems
                ui::update_drag_preview,
                ui::spawn_drag_preview,
                ui::cleanup_drag_preview,
            ));
    }
}
