//! Group/ungroup operations for hierarchical object management
//!
//! This module provides:
//! - Ctrl+G: Group selected entities under a new parent
//! - Ctrl+Shift+G: Ungroup selected groups (flatten hierarchy)
//! - Automatic group naming with incrementing numbers
//! - Transform hierarchy management (world ↔ local space conversion)

use bevy::prelude::*;
use crate::editor::objects::selection::SelectionSet;
use crate::editor::objects::placement::PlacementState;
use crate::editor::core::types::EditorEntity;

/// Resource to track group count for auto-incrementing group names
#[derive(Resource, Default)]
pub struct GroupCounter {
    pub count: usize,
}

impl GroupCounter {
    /// Get the next group name and increment the counter
    pub fn next_name(&mut self) -> String {
        if self.count == 0 {
            self.count += 1;
            "Group".to_string()
        } else {
            let name = format!("Group {}", self.count);
            self.count += 1;
            name
        }
    }
}

/// Marker component for entities that are groups (parents of grouped objects)
#[derive(Component)]
pub struct Group;

/// Handle Ctrl+G to group selected entities
pub fn handle_group(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    selection: Res<SelectionSet>,
    mut group_counter: ResMut<GroupCounter>,
    placement_state: Res<PlacementState>,
    entity_query: Query<&GlobalTransform, With<EditorEntity>>,
) {
    // Don't group if in placement mode
    if placement_state.active {
        return;
    }

    // Check for Ctrl+G (without Shift)
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    
    if ctrl_held && keyboard.just_pressed(KeyCode::KeyG) && !shift_held {
        // Need at least 2 entities to group
        if selection.len() < 2 {
            info!("Cannot group: need at least 2 selected entities");
            return;
        }

        // Calculate the center position of all selected entities (for group origin)
        let mut center = Vec3::ZERO;
        let mut valid_count = 0;
        
        for &entity in &selection.entities {
            if let Ok(global_transform) = entity_query.get(entity) {
                center += global_transform.translation();
                valid_count += 1;
            }
        }

        if valid_count == 0 {
            warn!("No valid transforms found for grouping");
            return;
        }

        center /= valid_count as f32;

        // Create the parent group entity
        let group_name = group_counter.next_name();
        let group_entity = commands
            .spawn((
                Name::new(group_name.clone()),
                EditorEntity,
                Group,
                Transform::from_translation(center),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .id();

        info!("Created group '{}' at position {:?}", group_name, center);

        // Convert each selected entity to be a child of the group
        // and convert their world transforms to local transforms relative to the group
        for &entity in &selection.entities {
            if let Ok(global_transform) = entity_query.get(entity) {
                // Calculate local transform relative to group center
                let world_pos = global_transform.translation();
                let world_rot = global_transform.to_scale_rotation_translation().1;
                let world_scale = global_transform.to_scale_rotation_translation().0;
                
                let local_pos = world_pos - center;
                
                // Update the entity's transform to be local and set parent
                commands.entity(entity).insert(ChildOf(group_entity));
                commands.entity(entity).insert(Transform {
                    translation: local_pos,
                    rotation: world_rot,
                    scale: world_scale,
                });

                info!("Added entity {:?} to group (local pos: {:?})", entity, local_pos);
            }
        }

        info!("Grouped {} entities under '{}'", selection.len(), group_name);
    }
}

/// Handle Ctrl+Shift+G to ungroup selected groups
pub fn handle_ungroup(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    selection: Res<SelectionSet>,
    placement_state: Res<PlacementState>,
    group_query: Query<&Children, With<Group>>,
    child_query: Query<(&GlobalTransform, &Transform)>,
) {
    // Don't ungroup if in placement mode
    if placement_state.active {
        return;
    }

    // Check for Ctrl+Shift+G
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    
    if ctrl_held && shift_held && keyboard.just_pressed(KeyCode::KeyG) {
        if selection.is_empty() {
            info!("Cannot ungroup: no entities selected");
            return;
        }

        let mut ungrouped_count = 0;

        // Process each selected entity
        for &entity in &selection.entities {
            // Check if this entity is a group
            if let Ok(children) = group_query.get(entity) {
                info!("Ungrouping entity {:?} with {} children", entity, children.len());

                // Promote each child to root level with world transforms
                for child in children.iter() {
                    if let Ok((global_transform, _local_transform)) = child_query.get(child) {
                        // Convert global transform to world-space transform
                        let world_pos = global_transform.translation();
                        let (world_scale, world_rot, _) = global_transform.to_scale_rotation_translation();

                        // Remove parent and set world transform
                        commands.entity(child).remove::<ChildOf>();
                        commands.entity(child).insert(Transform {
                            translation: world_pos,
                            rotation: world_rot,
                            scale: world_scale,
                        });

                        info!("Promoted child {:?} to root (world pos: {:?})", child, world_pos);
                    }
                }

                // Delete the now-empty group entity
                commands.entity(entity).despawn();
                ungrouped_count += 1;
            } else {
                // If selected entity is a child of a group, ungroup it individually
                // This allows ungrouping specific children without selecting the parent
                commands.entity(entity).remove::<ChildOf>();
                
                // Get the world transform and apply it
                if let Ok((global_transform, _)) = child_query.get(entity) {
                    let world_pos = global_transform.translation();
                    let (world_scale, world_rot, _) = global_transform.to_scale_rotation_translation();
                    
                    commands.entity(entity).insert(Transform {
                        translation: world_pos,
                        rotation: world_rot,
                        scale: world_scale,
                    });
                    
                    info!("Promoted single entity {:?} to root (world pos: {:?})", entity, world_pos);
                    ungrouped_count += 1;
                }
            }
        }

        if ungrouped_count > 0 {
            info!("Ungrouped {} groups/entities", ungrouped_count);
        } else {
            info!("No groups found in selection to ungroup");
        }
    }
}
