//! Raycasting utilities for viewport interaction
//!
//! This module provides raycasting functions used for determining where the cursor
//! points in 3D space. This is essential for object placement and other viewport
//! interactions.
//!
//! # Primary Use Case
//!
//! When placing objects, we need to know where on the ground plane (Y=0) the user
//! is pointing. This module provides `ray_plane_intersection()` to calculate that
//! intersection point from screen coordinates.

use bevy::prelude::*;

/// Ray-plane intersection helper
pub fn ray_plane_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    plane_origin: Vec3,
    plane_normal: Vec3,
) -> Option<f32> {
    let denom = plane_normal.dot(ray_direction);
    if denom.abs() < 0.0001 {
        return None; // Ray is parallel to plane
    }

    let t = (plane_origin - ray_origin).dot(plane_normal) / denom;
    if t >= 0.0 {
        Some(t)
    } else {
        None
    }
}
