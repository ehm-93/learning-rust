//! Viewport domain - 3D scene viewing and navigation
//!
//! This domain handles everything related to viewing and navigating the 3D editor scene:
//! - Camera controls (fly camera with WASD + mouse look)
//! - Grid rendering and snapping
//! - Ray-plane intersection utilities for object placement

pub mod camera;
pub mod grid;
pub mod plugin;
pub mod raycasting;

// Re-export plugin
pub use plugin::{ViewportPlugin, LightingEnabled, SavedLightingState};

// Re-export commonly used types for other modules
pub use camera::EditorCamera;
pub use grid::{GridConfig, snap_to_grid};
pub use raycasting::ray_plane_intersection;
