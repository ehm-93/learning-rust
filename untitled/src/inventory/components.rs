use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::inventory::registry::{ItemId, GridSize};

/// Unique identifier for item instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceId(pub u64);

/// Position within an inventory grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: u32,
    pub y: u32,
}

impl GridPosition {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

/// Rotation state for items that can be rotated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemRotation {
    None,        // 0°
    Clockwise90, // 90°
    Clockwise180,// 180°
    Clockwise270,// 270°
}

impl ItemRotation {
    /// Apply rotation to item dimensions
    pub fn apply_to_size(&self, size: GridSize) -> GridSize {
        match self {
            ItemRotation::None | ItemRotation::Clockwise180 => size,
            ItemRotation::Clockwise90 | ItemRotation::Clockwise270 => {
                GridSize::new(size.height, size.width)
            }
        }
    }

    /// Rotate 90 degrees clockwise
    pub fn rotate_clockwise(&self) -> Self {
        match self {
            ItemRotation::None => ItemRotation::Clockwise90,
            ItemRotation::Clockwise90 => ItemRotation::Clockwise180,
            ItemRotation::Clockwise180 => ItemRotation::Clockwise270,
            ItemRotation::Clockwise270 => ItemRotation::None,
        }
    }
}

/// Actual item instance with rolled properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInstance {
    /// Unique instance identifier
    pub id: InstanceId,
    /// Reference to item definition
    pub item_id: ItemId,
    /// Current stack size (1 if not stackable)
    pub stack_size: u32,
    /// Position in inventory grid
    pub position: GridPosition,
    /// Current rotation
    pub rotation: ItemRotation,
    /// Rolled property values
    pub properties: HashMap<String, f32>,
    /// Boolean flags
    pub flags: HashMap<String, bool>,
    /// String properties
    pub strings: HashMap<String, String>,
    /// Durability (if applicable)
    pub durability: Option<f32>,
}

impl ItemInstance {
    pub fn new(id: InstanceId, item_id: ItemId) -> Self {
        Self {
            id,
            item_id,
            stack_size: 1,
            position: GridPosition::zero(),
            rotation: ItemRotation::None,
            properties: HashMap::new(),
            flags: HashMap::new(),
            strings: HashMap::new(),
            durability: None,
        }
    }

    /// Get a numeric property value
    pub fn get_property(&self, name: &str) -> Option<f32> {
        self.properties.get(name).copied()
    }

    /// Get a boolean flag
    pub fn get_flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }

    /// Get a string property
    pub fn get_string(&self, name: &str) -> Option<&String> {
        self.strings.get(name)
    }
}

/// Individual cell in the inventory grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    /// Instance ID occupying this cell
    pub item_id: InstanceId,
    /// Whether this is the origin cell (top-left) for multi-cell items
    pub is_origin: bool,
}

/// Inventory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryConfig {
    /// Base grid width
    pub base_width: u32,
    /// Base grid height
    pub base_height: u32,
    /// Current width (can be modified)
    pub current_width: u32,
    /// Current height (can be modified)
    pub current_height: u32,
    /// Whether items can be rotated in this inventory
    pub allow_rotation: bool,
    /// Whether items can stack automatically
    pub allow_stacking: bool,
    /// Whether inventory can auto-sort
    pub allow_auto_sort: bool,
}

impl InventoryConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            base_width: width,
            base_height: height,
            current_width: width,
            current_height: height,
            allow_rotation: true,
            allow_stacking: true,
            allow_auto_sort: false,
        }
    }

    /// Get total current capacity
    pub fn capacity(&self) -> u32 {
        self.current_width * self.current_height
    }

    /// Modify current size (for dynamic inventory sizing)
    pub fn resize(&mut self, width: u32, height: u32) {
        self.current_width = width;
        self.current_height = height;
    }

    /// Reset to base size
    pub fn reset_size(&mut self) {
        self.current_width = self.base_width;
        self.current_height = self.base_height;
    }
}

/// 2D grid for inventory storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryGrid {
    /// 2D array of grid cells
    pub cells: Vec<Vec<Option<GridCell>>>,
    /// All item instances in this inventory
    pub items: HashMap<InstanceId, ItemInstance>,
    /// Configuration
    pub config: InventoryConfig,
}

impl InventoryGrid {
    pub fn new(config: InventoryConfig) -> Self {
        let mut cells = Vec::with_capacity(config.current_height as usize);
        for _ in 0..config.current_height {
            let mut row = Vec::with_capacity(config.current_width as usize);
            for _ in 0..config.current_width {
                row.push(None);
            }
            cells.push(row);
        }

        Self {
            cells,
            items: HashMap::new(),
            config,
        }
    }

    /// Check if a position is within grid bounds
    pub fn is_valid_position(&self, pos: GridPosition) -> bool {
        pos.x < self.config.current_width && pos.y < self.config.current_height
    }

    /// Check if an area is free for item placement
    pub fn is_area_free(&self, pos: GridPosition, size: GridSize) -> bool {
        // Check if entire area fits within grid
        if pos.x + size.width > self.config.current_width
            || pos.y + size.height > self.config.current_height
        {
            return false;
        }

        // Check if all cells in area are empty
        for y in pos.y..(pos.y + size.height) {
            for x in pos.x..(pos.x + size.width) {
                if self.cells[y as usize][x as usize].is_some() {
                    return false;
                }
            }
        }

        true
    }

    /// Get item at specific position
    pub fn get_item_at(&self, pos: GridPosition) -> Option<&ItemInstance> {
        if !self.is_valid_position(pos) {
            return None;
        }

        let cell = &self.cells[pos.y as usize][pos.x as usize];
        cell.as_ref().and_then(|cell| self.items.get(&cell.item_id))
    }

    /// Resize the grid (for dynamic inventory sizing)
    pub fn resize(&mut self, new_width: u32, new_height: u32) -> Result<(), String> {
        // Check if any items would be cut off
        for (_, item) in &self.items {
            if item.position.x >= new_width || item.position.y >= new_height {
                return Err(format!(
                    "Cannot resize: item at ({}, {}) would be outside new bounds",
                    item.position.x, item.position.y
                ));
            }
        }

        // Resize the grid
        self.cells.resize(new_height as usize, Vec::new());
        for row in &mut self.cells {
            row.resize(new_width as usize, None);
        }

        self.config.resize(new_width, new_height);
        Ok(())
    }
}

/// Main inventory component that can be attached to any entity
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub grid: InventoryGrid,
}

impl Inventory {
    pub fn new(width: u32, height: u32) -> Self {
        let config = InventoryConfig::new(width, height);
        Self {
            grid: InventoryGrid::new(config),
        }
    }

    /// Create a player inventory with standard settings
    pub fn player_inventory() -> Self {
        let mut config = InventoryConfig::new(6, 4);
        config.allow_rotation = true;
        config.allow_stacking = true;
        config.allow_auto_sort = false;

        Self {
            grid: InventoryGrid::new(config),
        }
    }

    /// Create a chest inventory
    pub fn chest_inventory(size: InventorySize) -> Self {
        let (width, height) = match size {
            InventorySize::Small => (4, 3),
            InventorySize::Medium => (6, 4),
            InventorySize::Large => (8, 6),
        };

        let mut config = InventoryConfig::new(width, height);
        config.allow_auto_sort = true;

        Self {
            grid: InventoryGrid::new(config),
        }
    }
}

/// Standard inventory sizes
pub enum InventorySize {
    Small,
    Medium,
    Large,
}
