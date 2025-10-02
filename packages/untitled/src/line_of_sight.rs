use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::{
    constants::*,
};

/// Fast line of sight check using Rapier's raycasting
/// Returns true if no walls block the path between start and end
/// The start_radius and end_radius parameters should be the collision radii of the entities
pub fn has_line_of_sight(
    start: Vec2,
    end: Vec2,
    start_radius: f32,
    end_radius: f32,
    rapier_context: &ReadRapierContext,
) -> bool {
    let direction = end - start;
    let distance = direction.length();

    if distance > LOS_MAX_RANGE {
        return false;
    }

    let ray_dir = direction.normalize();
    let buffer = 2.0;

    // Calculate total offset needed (both entities' radii plus buffers)
    let total_offset = start_radius + end_radius + (buffer * 2.0);

    if distance <= total_offset {
        return true; // Too close to raycast meaningfully, assume visible
    }

    // Start the ray outside the start entity's collision radius
    let ray_start = start + ray_dir * (start_radius + buffer);

    // Calculate remaining distance, stopping before the end entity's collision radius
    let remaining_distance = distance - (start_radius + buffer) - (end_radius + buffer);

    if remaining_distance <= 0.0 {
        return true; // Too close to raycast meaningfully
    }

    // Create a filter that excludes sensors (projectiles) but includes solid objects (walls)
    let filter = QueryFilter::default().exclude_sensors();

    // Get the rapier context and perform raycast
    if let Ok(context) = rapier_context.single() {
        // Perform the raycast using Rapier's efficient raycasting
        if let Some((_entity, _toi)) = context.cast_ray(
            ray_start,
            ray_dir,
            remaining_distance,
            true, // solid objects only
            filter,
        ) {
            // Ray hit something solid before reaching the target
            false
        } else {
            // Clear line of sight
            true
        }
    } else {
        // Fallback: assume no line of sight if we can't access Rapier context
        println!("Warning: Unable to access Rapier context for line of sight check.");
        false
    }
}// The update_line_of_sight system has been moved into the enemy_ai system for better efficiency
