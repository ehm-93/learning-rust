//! Duplicate/delete operations for object management
//!
//! This module provides:
//! - Ctrl+D: Duplicate selected entities with +1m X-axis offset
//! - Maintains parent-child relationships
//! - Clones all components (Transform, Mesh3d, MeshMaterial3d, Name, etc.)
//! - Auto-selects duplicated objects after creation

use bevy::prelude::*;
use crate::editor::objects::selection::{SelectionSet, Selected};
use crate::editor::objects::placement::PlacementState;
use crate::editor::objects::grouping::Group;
use crate::editor::core::types::{EditorEntity, GlbModel, RigidBodyType, PlayerSpawn};
use std::collections::HashMap;

/// Handle Ctrl+D to duplicate selected entities
pub fn handle_duplicate(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selection: ResMut<SelectionSet>,
    placement_state: Res<PlacementState>,
    selected_query: Query<Entity, With<Selected>>,
    // Query all components separately to avoid hitting Bevy's parameter limit
    transform_query: Query<&Transform, With<EditorEntity>>,
    name_query: Query<&Name>,
    mesh_query: Query<&Mesh3d>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    group_query: Query<(), With<Group>>,
    glb_query: Query<&GlbModel>,
    rb_query: Query<&RigidBodyType>,
    visibility_query: Query<&Visibility>,
    parent_query: Query<&ChildOf>,
    player_spawn_query: Query<(), With<PlayerSpawn>>,
) {
    // Don't duplicate if in placement mode
    if placement_state.active {
        return;
    }

    // Check for Ctrl+D
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    
    if ctrl_held && keyboard.just_pressed(KeyCode::KeyD) {
        if selection.is_empty() {
            info!("Cannot duplicate: no entities selected");
            return;
        }

        // Calculate the dominant horizontal axis for offset
        // For now, always use X-axis (+1.0 offset)
        let offset = Vec3::new(1.0, 0.0, 0.0);

        // Map from original entity to cloned entity (for parent-child relationships)
        let mut entity_map: HashMap<Entity, Entity> = HashMap::new();
        
        // Collect selected entities into a vec to avoid borrow conflicts
        let selected_entities: Vec<Entity> = selected_query.iter().collect();

        // First pass: Clone all entities without setting up parent-child relationships
        for &original_entity in &selected_entities {
            // Get the transform (required)
            let Ok(transform) = transform_query.get(original_entity) else {
                warn!("Entity {:?} has no transform, skipping", original_entity);
                continue;
            };

            // Create new transform with offset
            let new_transform = Transform {
                translation: transform.translation + offset,
                rotation: transform.rotation,
                scale: transform.scale,
            };

            // Start building the entity with required components
            let mut entity_commands = commands.spawn((
                EditorEntity,
                new_transform,
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Clone Name (with " Copy" suffix if it exists)
            if let Ok(original_name) = name_query.get(original_entity) {
                let new_name = format!("{} Copy", original_name.as_str());
                entity_commands.insert(Name::new(new_name));
            } else {
                entity_commands.insert(Name::new("Copy"));
            }

            // Clone Mesh3d if present
            if let Ok(mesh) = mesh_query.get(original_entity) {
                entity_commands.insert(mesh.clone());
            }

            // Clone Material if present
            if let Ok(material) = material_query.get(original_entity) {
                entity_commands.insert(material.clone());
            }

            // Clone Group marker if present
            if group_query.get(original_entity).is_ok() {
                entity_commands.insert(Group);
            }

            // Clone GlbModel if present
            if let Ok(glb) = glb_query.get(original_entity) {
                entity_commands.insert(glb.clone());
            }

            // Clone RigidBodyType if present
            if let Ok(rb_type) = rb_query.get(original_entity) {
                entity_commands.insert(*rb_type);
            }

            // Clone Visibility if present
            if let Ok(vis) = visibility_query.get(original_entity) {
                entity_commands.insert(*vis);
            }

            // Clone PlayerSpawn marker if present
            if player_spawn_query.get(original_entity).is_ok() {
                entity_commands.insert(PlayerSpawn);
            }

            let cloned_entity = entity_commands.id();
            entity_map.insert(original_entity, cloned_entity);

            info!("Duplicated entity {:?} -> {:?}", original_entity, cloned_entity);
        }

        // Second pass: Set up parent-child relationships using the entity map
        for &original_entity in &selected_entities {
            let cloned_entity = entity_map[&original_entity];

            // If this entity had a parent that was also selected, maintain the relationship
            if let Ok(child_of) = parent_query.get(original_entity) {
                if let Some(&cloned_parent) = entity_map.get(&child_of.0) {
                    commands.entity(cloned_entity).insert(ChildOf(cloned_parent));
                    info!("Set parent of {:?} to {:?}", cloned_entity, cloned_parent);
                }
            }
        }

        // Clear previous selection and select the duplicated entities
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        selection.clear();

        // Select all duplicated entities
        for &cloned_entity in entity_map.values() {
            selection.add(cloned_entity);
            commands.entity(cloned_entity).insert(Selected);
        }

        info!("Duplicated {} entities with +1m X-axis offset", entity_map.len());
    }
}
