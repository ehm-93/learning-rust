//! Gizmo types and components
//!
//! This module defines the data structures used by the gizmo system:
//! - Transform modes (translate, rotate, scale)
//! - Transform orientations (global, local)
//! - Gizmo components and markers

use bevy::prelude::*;

/// Transform mode for the gizmo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

/// Transform orientation (coordinate space) for the gizmo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformOrientation {
    /// Gizmo aligns with world axes (XYZ)
    #[default]
    Global,
    /// Gizmo aligns with object's local axes
    Local,
}

/// Resource tracking current transform mode and orientation
#[derive(Resource)]
pub struct GizmoState {
    pub mode: TransformMode,
    pub orientation: TransformOrientation,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            mode: TransformMode::Translate,
            orientation: TransformOrientation::Global,
        }
    }
}

/// Axis identifier for gizmo handles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

/// Marker component for gizmo handle entities (the interactive parts)
#[derive(Component)]
pub struct GizmoHandle;

/// Marker component for the gizmo root entity
#[derive(Component)]
pub struct GizmoRoot;
