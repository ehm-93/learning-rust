use bevy::prelude::*;
use bevy::tasks::Task;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use crate::world::chunks::{CHUNK_SIZE};

/// Component for entities that reveal fog of war around them
#[derive(Component, Clone, PartialEq, Debug)]
pub struct FowRevealer {
    /// Radius at which all tiles are exposed regardless of line of sight
    pub radius: usize,
    /// Strength of the forced reveal area (0-1)
    pub strength: f32,
    /// Radius at which line of sight is calculated
    pub los_radius: usize,
    /// Strength of the line of sight reveal area (0-1)
    pub los_strength: f32,
}

impl FowRevealer {
    pub fn new(radius: usize, los_radius: usize) -> Self {
        Self {
            radius,
            los_radius,
            ..default()
        }
    }

    pub fn with_strength(radius: usize, strength: f32, los_radius: usize, los_strength: f32) -> Self {
        Self {
            radius,
            strength,
            los_radius,
            los_strength,
        }
    }
}

impl Default for FowRevealer {
    fn default() -> Self {
        Self {
            radius: 12,     // 12 tiles always visible
            strength: 1.0,
            los_radius: 64,      // 64 tiles with line of sight
            los_strength: 0.5,
        }
    }
}

/// Marker component for FOW chunks that need lerping (have mismatched current vs desired vision)
#[derive(Component)]
pub struct NeedsLerp;

/// Fog of war chunk data with smooth interpolation support
#[derive(Component, Clone, Debug)]
pub struct FowChunk {
    /// The position of the chunk in chunk coordinates
    pub position: IVec2,
    /// Current interpolated vision state (0 = fogged, 255 = fully revealed)
    pub current_vision: Vec<Vec<f32>>,
    /// Target vision state from background calculations
    pub desired_vision: Vec<Vec<u8>>,
    /// Speed at which current_vision lerps toward desired_vision (0-1, higher = faster)
    pub lerp_speed: f32,
}

impl FowChunk {
    pub fn new(position: IVec2, size: usize) -> Self {
        Self {
            position,
            current_vision: vec![vec![0.0; size]; size],
            desired_vision: vec![vec![0u8; size]; size],
            lerp_speed: 0.05, // Smooth but responsive transition
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

/// Work item for background FOW calculation
#[derive(Clone, Debug)]
pub struct FowWorkItem {
    /// Position of the revealer in world coordinates
    pub revealer_pos: Vec2,
    /// Force radius (always visible)
    pub force_radius: usize,
    /// Force strength
    pub force_strength: f32,
    /// LOS radius
    pub los_radius: usize,
    /// LOS strength
    pub los_strength: f32,
    /// Terrain data for line-of-sight calculations (chunk_pos -> tiles)
    pub terrain_snapshot: Arc<HashMap<IVec2, Vec<Vec<bool>>>>,
}

/// Result from background FOW calculation
#[derive(Clone, Debug)]
pub struct FowWorkResult {
    /// Map of chunk positions to their updated vision data
    pub chunk_updates: HashMap<IVec2, Vec<Vec<u8>>>,
}

/// A single FOW calculation task
#[derive(Component)]
pub struct FowCalculationTask {
    pub task: Task<FowWorkResult>,
    pub revealer_entity: Entity,
}

/// Resource for managing active FOW calculation tasks
#[derive(Resource, Default)]
pub struct FowTaskSet {
    /// Set of entities that have active FOW calculation tasks
    pub active_tasks: HashSet<Entity>,
}

