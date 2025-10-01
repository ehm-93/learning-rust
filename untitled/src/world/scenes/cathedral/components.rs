use bevy::prelude::*;

/// Marker component for the Cathedral scene
#[derive(Component)]
pub struct Cathedral;

/// Portal component representing different scene entry points
#[derive(Component)]
pub struct Portal {
    pub id: PortalId,
    pub portal_type: PortalType,
    pub depth: u32,
    pub modifiers: Vec<ModifierId>,
}

/// Portal identifier
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PortalId {
    DungeonLeft,
    DungeonCenter,
    DungeonRight,
}

/// Portal destination type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PortalType {
    Dungeon,
}

/// Modifier identifier for dungeon modifications
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ModifierId {
    // Basic modifiers for Phase 1a
    EnemyDensity(i8),        // -3 to +3 (fewer to more enemies)
    EnemySpeed(i8),          // -3 to +3 (slower to faster)
    RoomSize(i8),            // -3 to +3 (smaller to larger rooms)
    LootQuantity(i8),        // -3 to +3 (less to more loot)
    // More modifiers will be added in later phases
}

impl ModifierId {
    /// Get the display name for this modifier
    pub fn display_name(&self) -> String {
        match self {
            ModifierId::EnemyDensity(value) => {
                match *value {
                    -3 => "Sparse Enemies".to_string(),
                    -2 => "Few Enemies".to_string(),
                    -1 => "Reduced Enemies".to_string(),
                    0 => "Normal Enemies".to_string(),
                    1 => "More Enemies".to_string(),
                    2 => "Dense Enemies".to_string(),
                    3 => "Swarming Enemies".to_string(),
                    _ => format!("Enemy Density {}", value),
                }
            },
            ModifierId::EnemySpeed(value) => {
                match *value {
                    -3 => "Sluggish Enemies".to_string(),
                    -2 => "Slow Enemies".to_string(),
                    -1 => "Lethargic Enemies".to_string(),
                    0 => "Normal Speed".to_string(),
                    1 => "Quick Enemies".to_string(),
                    2 => "Fast Enemies".to_string(),
                    3 => "Lightning Enemies".to_string(),
                    _ => format!("Enemy Speed {}", value),
                }
            },
            ModifierId::RoomSize(value) => {
                match *value {
                    -3 => "Cramped Rooms".to_string(),
                    -2 => "Small Rooms".to_string(),
                    -1 => "Compact Rooms".to_string(),
                    0 => "Normal Rooms".to_string(),
                    1 => "Large Rooms".to_string(),
                    2 => "Spacious Rooms".to_string(),
                    3 => "Vast Rooms".to_string(),
                    _ => format!("Room Size {}", value),
                }
            },
            ModifierId::LootQuantity(value) => {
                match *value {
                    -3 => "Barren Loot".to_string(),
                    -2 => "Scarce Loot".to_string(),
                    -1 => "Sparse Loot".to_string(),
                    0 => "Normal Loot".to_string(),
                    1 => "Rich Loot".to_string(),
                    2 => "Abundant Loot".to_string(),
                    3 => "Overflowing Loot".to_string(),
                    _ => format!("Loot Quantity {}", value),
                }
            },
        }
    }

    /// Get a random modifier of the given type
    pub fn random_of_type(modifier_type: &str, rng: &mut impl rand::Rng) -> Option<Self> {
        let value = rng.random_range(-2..=2); // Start with moderate values for Phase 1a
        match modifier_type {
            "enemy_density" => Some(ModifierId::EnemyDensity(value)),
            "enemy_speed" => Some(ModifierId::EnemySpeed(value)),
            "room_size" => Some(ModifierId::RoomSize(value)),
            "loot_quantity" => Some(ModifierId::LootQuantity(value)),
            _ => None,
        }
    }
}

/// Component for portal UI display text
#[derive(Component)]
pub struct PortalDisplay {
    pub portal_id: PortalId,
}
