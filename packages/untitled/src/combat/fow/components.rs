use bevy::prelude::*;

use crate::world::chunks::{CHUNK_SIZE};

/// Component for entities that reveal fog of war around them
#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub struct FowRevealer {
    /// Vision radius in tiles
    pub radius: usize,
}

impl FowRevealer {
    pub fn new(radius: usize) -> Self {
        Self { radius }
    }
}

impl Default for FowRevealer {
    fn default() -> Self {
        Self { radius: 10 } // 10 tiles vision radius
    }
}

/// Fog of war chunk data
#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub struct FowChunk {
    /// The position of the chunk in chunk coordinates
    pub position: IVec2,
    /// 2D grid representing vision in the chunk (0 = fogged, 255 = fully revealed)
    pub vision: Vec<Vec<u8>>,
}

impl FowChunk {
    pub fn new(position: IVec2, size: usize) -> Self {
        Self {
            position,
            vision: vec![vec![0u8; size]; size],
        }
    }
}

impl Default for FowChunk {
    fn default() -> Self {
        Self::new(IVec2::ZERO, CHUNK_SIZE as usize)
    }
}

/// Marker component for the fog of war overlay entity
#[derive(Component, Clone, PartialEq, Eq, Debug)]
pub struct FowOverlay {
    /// The position of the overlay in chunk coordinates
    pub position: IVec2,
}

impl FowOverlay {
    pub fn new(position: IVec2) -> Self {
        Self { position }
    }
}

impl Default for FowOverlay {
    fn default() -> Self {
        Self::new(IVec2::ZERO)
    }
}

