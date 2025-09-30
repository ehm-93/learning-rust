use bevy::prelude::*;
use crate::inventory::{InstanceId, GridPosition};

/// Events for inventory interactions
#[derive(Event)]
pub enum InventoryEvent {
    /// An item was selected in the inventory
    ItemSelected {
        item_id: InstanceId,
    },
    /// An item was deselected
    ItemDeselected {
        item_id: InstanceId,
    },
    /// An attempt to move an item to a new position
    ItemMoveAttempt {
        item_id: InstanceId,
        target_position: GridPosition,
    },
    /// An item was successfully moved
    ItemMoved {
        item_id: InstanceId,
        from_position: GridPosition,
        to_position: GridPosition,
    },
    /// An item was used/consumed
    ItemUsed {
        item_id: InstanceId,
    },
    /// Inventory panel was opened
    InventoryOpened,
    /// Inventory panel was closed
    InventoryClosed,
}

/// Events for UI state changes
#[derive(Event)]
pub struct TooltipEvent {
    pub item_id: Option<InstanceId>,
    pub show: bool,
}
