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
