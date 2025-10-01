use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use super::components::{PortalId, ModifierId};

/// Central resource for managing Cathedral state
#[derive(Resource, Default, Clone)]
pub struct CathedralState {
    pub is_active: bool,
    pub current_depth: u32,
}

/// Resource for managing the modifier system and portal configurations
#[derive(Resource, Clone)]
pub struct ModifierSystem {
    /// Stable modifier combinations for each portal at each depth
    /// Key: (depth, portal_id) -> Vec<ModifierId>
    pub portal_configurations: HashMap<(u32, PortalId), Vec<ModifierId>>,
    /// Available modifier types for random generation
    pub available_modifier_types: Vec<String>,
}

impl Default for ModifierSystem {
    fn default() -> Self {
        Self {
            portal_configurations: HashMap::new(),
            available_modifier_types: vec![
                "enemy_density".to_string(),
                "enemy_speed".to_string(),
                "room_size".to_string(),
                "loot_quantity".to_string(),
            ],
        }
    }
}

impl ModifierSystem {
    /// Create a new ModifierSystem with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate stable modifier combinations for a specific depth level
    pub fn generate_portal_modifiers(&mut self, depth: u32, rng: &mut impl Rng) {
        for portal_id in [PortalId::DungeonLeft, PortalId::DungeonCenter, PortalId::DungeonRight] {
            // Only generate if we don't already have modifiers for this portal/depth combo
            if !self.portal_configurations.contains_key(&(depth, portal_id)) {
                let mut modifiers = Vec::new();

                // Generate 2-3 modifiers per portal for variety
                let modifier_count = rng.random_range(2..=3);
                let mut used_types = Vec::new();

                for _ in 0..modifier_count {
                    // Pick a random modifier type we haven't used yet
                    let available_types: Vec<_> = self.available_modifier_types
                        .iter()
                        .filter(|t| !used_types.contains(*t))
                        .collect();

                    if !available_types.is_empty() {
                        let index = rng.random_range(0..available_types.len());
                        let modifier_type = available_types[index];
                        if let Some(modifier) = ModifierId::random_of_type(modifier_type, rng) {
                            modifiers.push(modifier);
                            used_types.push(modifier_type.to_string());
                        }
                    }
                }

                self.portal_configurations.insert((depth, portal_id), modifiers);
            }
        }
    }

    /// Get the stable modifier combination for a specific portal and depth
    pub fn get_portal_modifiers(&self, depth: u32, portal_id: PortalId) -> Vec<ModifierId> {
        self.portal_configurations
            .get(&(depth, portal_id))
            .cloned()
            .unwrap_or_default()
    }

    /// Reroll modifiers for all portals at a given depth (costs currency in later phases)
    pub fn reroll_depth_modifiers(&mut self, depth: u32, rng: &mut impl Rng) {
        // Remove existing configurations for this depth
        self.portal_configurations.retain(|(d, _), _| *d != depth);

        // Generate new ones
        self.generate_portal_modifiers(depth, rng);
    }
}

/// Resource for tracking player progression and unlocked depths
#[derive(Resource, Default, Clone)]
pub struct ProgressionState {
    /// Maximum depth the player has successfully extracted from
    pub max_extracted_depth: u32,
    /// Currently unlocked starting depths (shortcuts)
    pub unlocked_depths: Vec<u32>,
}

impl ProgressionState {
    /// Check if a depth is unlocked for direct portal access
    pub fn is_depth_unlocked(&self, depth: u32) -> bool {
        depth == 1 || self.unlocked_depths.contains(&depth)
    }

    /// Unlock a new starting depth based on successful extraction
    pub fn unlock_depth_from_extraction(&mut self, extracted_from_depth: u32) {
        self.max_extracted_depth = self.max_extracted_depth.max(extracted_from_depth);

        // Unlock shortcuts: can start at k*10+1 after successfully extracting from k*10+5
        // But cap shortcuts at level 100
        for k in 1..=10 {
            let unlock_threshold = k * 10 + 5;
            let unlock_depth = k * 10 + 1;

            if extracted_from_depth >= unlock_threshold && unlock_depth <= 100 {
                if !self.unlocked_depths.contains(&unlock_depth) {
                    self.unlocked_depths.push(unlock_depth);
                }
            }
        }

        // Sort unlocked depths for display
        self.unlocked_depths.sort();
    }

    /// Get the available starting depths for portal display
    pub fn get_available_depths(&self) -> Vec<u32> {
        let mut depths = vec![1]; // Level 1 always available
        depths.extend(&self.unlocked_depths);
        depths.sort();
        depths.dedup();
        depths
    }
}
