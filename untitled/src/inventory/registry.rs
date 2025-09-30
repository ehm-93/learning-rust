use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for item types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u32);

/// Size of an item in inventory grid units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridSize {
    pub width: u32,
    pub height: u32,
}

impl GridSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Standard 1x1 item size
    pub fn single() -> Self {
        Self::new(1, 1)
    }
}

/// Property value that can be randomized within a range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyRange {
    /// Fixed value, no randomization
    Fixed(f32),
    /// Range with min and max values
    Range { min: f32, max: f32 },
    /// Base value with percentage variance (e.g., 100 ±15%)
    Variance { base: f32, percent: f32 },
}

impl PropertyRange {
    /// Generate a random value within this range
    pub fn roll(&self, rng: &mut impl rand::Rng) -> f32 {
        match self {
            PropertyRange::Fixed(value) => *value,
            PropertyRange::Range { min, max } => rng.random_range(*min..=*max),
            PropertyRange::Variance { base, percent } => {
                let variance = base * (percent / 100.0);
                rng.random_range((base - variance)..=(base + variance))
            }
        }
    }
}

/// Generic properties that can be attached to items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemProperties {
    /// Numeric properties with randomizable ranges
    pub numeric: HashMap<String, PropertyRange>,
    /// Boolean flags
    pub flags: HashMap<String, bool>,
    /// String properties
    pub strings: HashMap<String, String>,
}

impl Default for ItemProperties {
    fn default() -> Self {
        Self {
            numeric: HashMap::new(),
            flags: HashMap::new(),
            strings: HashMap::new(),
        }
    }
}

/// Template for creating item instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDefinition {
    /// Unique identifier
    pub id: ItemId,
    /// Display name
    pub name: String,
    /// Description text
    pub description: String,
    /// Item category for organization
    pub category: String,
    /// Size in inventory grid
    pub size: GridSize,
    /// Maximum stack size (None = not stackable)
    pub max_stack_size: Option<u32>,
    /// Whether this item can be rotated
    pub can_rotate: bool,
    /// Properties with potential randomization
    pub properties: ItemProperties,
    /// Path to item icon
    pub icon_path: String,
}

impl ItemDefinition {
    pub fn new(id: ItemId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: String::new(),
            category: "misc".to_string(),
            size: GridSize::single(),
            max_stack_size: None,
            can_rotate: false,
            properties: ItemProperties::default(),
            icon_path: String::new(),
        }
    }

    /// Builder pattern methods for easy creation
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = GridSize::new(width, height);
        self
    }

    pub fn with_stack_size(mut self, max_stack: u32) -> Self {
        self.max_stack_size = Some(max_stack);
        self
    }

    pub fn rotatable(mut self) -> Self {
        self.can_rotate = true;
        self
    }

    pub fn with_property(mut self, name: impl Into<String>, range: PropertyRange) -> Self {
        self.properties.numeric.insert(name.into(), range);
        self
    }

    pub fn with_flag(mut self, name: impl Into<String>, value: bool) -> Self {
        self.properties.flags.insert(name.into(), value);
        self
    }
}

/// Global registry of all item definitions
#[derive(Resource, Default)]
pub struct ItemRegistry {
    /// All item definitions indexed by ID
    pub items: HashMap<ItemId, ItemDefinition>,
    /// Items grouped by category
    pub categories: HashMap<String, Vec<ItemId>>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new item definition
    pub fn register(&mut self, definition: ItemDefinition) {
        let category = definition.category.clone();
        let id = definition.id;

        // Add to main registry
        self.items.insert(id, definition);

        // Add to category index
        self.categories.entry(category).or_insert_with(Vec::new).push(id);
    }

    /// Get item definition by ID
    pub fn get(&self, id: ItemId) -> Option<&ItemDefinition> {
        self.items.get(&id)
    }

    /// Get all items in a category
    pub fn get_category(&self, category: &str) -> Vec<&ItemDefinition> {
        self.categories
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.items.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all registered item IDs
    pub fn all_ids(&self) -> Vec<ItemId> {
        self.items.keys().copied().collect()
    }
}

/// System to initialize the item registry with default items
pub fn setup_item_registry(mut commands: Commands) {
    let mut registry = ItemRegistry::new();

    // Example item definitions - these would normally be loaded from data files
    registry.register(
        ItemDefinition::new(ItemId(1), "Health Potion")
            .with_description("Restores health when consumed")
            .with_category("consumable")
            .with_stack_size(10)
            .with_property("heal_amount", PropertyRange::Variance { base: 50.0, percent: 20.0 })
    );

    registry.register(
        ItemDefinition::new(ItemId(2), "Basic Rifle")
            .with_description("A reliable firearm")
            .with_category("weapon")
            .with_size(3, 1)
            .rotatable()
            .with_property("damage", PropertyRange::Range { min: 25.0, max: 35.0 })
            .with_property("fire_rate", PropertyRange::Variance { base: 2.0, percent: 10.0 })
    );

    registry.register(
        ItemDefinition::new(ItemId(3), "Armor Vest")
            .with_description("Provides protection from damage")
            .with_category("armor")
            .with_size(2, 2)
            .with_property("armor", PropertyRange::Range { min: 15.0, max: 25.0 })
            .with_property("durability", PropertyRange::Fixed(100.0))
    );

    commands.insert_resource(registry);
}
