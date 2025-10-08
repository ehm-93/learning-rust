//! MapId - Unique identifier for map instances
//!
//! This module provides a MapId type that uniquely identifies a map instance
//! (e.g., a specific dungeon run). This prevents database collisions between
//! different map instances by using the generation seed as the unique identifier.

use bevy::prelude::*;
use std::fmt;

/// Unique identifier for a map instance
///
/// MapId is used to namespace chunk data in the database, preventing collisions
/// between different dungeon runs. Each dungeon run has a unique seed that serves
/// as its identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct MapId {
    /// Generation seed that uniquely identifies this map instance
    seed: u64,
}

impl MapId {
    /// Create a new MapId from a seed
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }
    
    /// Get the generation seed
    pub fn seed(&self) -> u64 {
        self.seed
    }
    
    /// Convert to i64 for database storage
    pub fn to_db_key(&self) -> i64 {
        self.seed as i64
    }
}

impl fmt::Display for MapId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MapId(seed:{})", self.seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_id_uniqueness() {
        let id1 = MapId::new(12345);
        let id2 = MapId::new(12345);
        let id3 = MapId::new(54321);
        
        // Same seed produces same ID
        assert_eq!(id1, id2);
        assert_eq!(id1.to_db_key(), id2.to_db_key());
        
        // Different seed produces different ID
        assert_ne!(id1, id3);
        assert_ne!(id1.to_db_key(), id3.to_db_key());
    }
    
    #[test]
    fn test_db_key_conversion() {
        let id = MapId::new(12345);
        assert_eq!(id.to_db_key(), 12345i64);
        
        let id2 = MapId::new(u64::MAX);
        assert_eq!(id2.to_db_key(), -1i64); // u64::MAX wraps to -1 as i64
    }
}
