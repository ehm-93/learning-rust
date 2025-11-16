//! Duplicate/delete operations for object management
//!
//! This module provides:
//! - Ctrl+D: Duplicate selected entities with +1m X-axis offset
//! - Maintains parent-child relationships
//! - Clones all components (Transform, Mesh3d, MeshMaterial3d, Name, etc.)
//! - Auto-selects duplicated objects after creation

//! Object duplication system
//!
//! This module handles duplicating selected objects. It creates exact copies
//! of selected entities with all their components, slightly offset from the
//! original position for visibility.
//!
//! # Features
//!
//! - **Component cloning**: Copies all serializable components (Transform, Mesh, Material, etc.)
//! - **Automatic offset**: New objects appear slightly offset (+1 unit on X axis)
//! - **Multi-select support**: Can duplicate multiple objects at once
//! - **Keyboard shortcut**: Ctrl+D to duplicate selection
//!
//! # Limitations
//!
//! Currently uses a manual list of components to copy. Future improvement:
//! use Bevy reflection system for automatic component cloning.

use bevy::prelude::*;
use crate::editor::objects::selection::{SelectionSet, Selected};
use crate::editor::objects::placement::PlacementState;
use crate::editor::objects::grouping::Group;
use crate::editor::core::types::{EditorEntity, GlbModel, RigidBodyType, PlayerSpawn, EditorLight, EditorVisualization};
use std::collections::HashMap;

/// Handle Ctrl+D to duplicate selected entities
pub fn handle_duplicate(world: &mut World) {
    // Check if in placement mode
    let placement_state = world.resource::<PlacementState>();
    if placement_state.active {
        return;
    }

    // Check for Ctrl+D keyboard input
    let keyboard = world.resource::<ButtonInput<KeyCode>>();
    let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if !ctrl_held || !keyboard.just_pressed(KeyCode::KeyD) {
        return;
    }

    // Check if there are selected entities
    let selection = world.resource::<SelectionSet>();
    if selection.is_empty() {
        info!("Cannot duplicate: no entities selected");
        return;
    }

    // Collect selected entities
    let mut selected_query = world.query_filtered::<Entity, With<Selected>>();
    let selected_entities: Vec<Entity> = selected_query.iter(world).collect();

    // Calculate offset for duplicated entities
    let offset = Vec3::new(1.0, 0.0, 0.0);

    // Map from original entity to cloned entity
    let mut entity_map: HashMap<Entity, Entity> = HashMap::new();

    // Collect all component data first to avoid borrow conflicts
    let mut entities_data = Vec::new();

    for &original_entity in &selected_entities {
        // Get transform (required)
        let Some(transform) = world.get::<Transform>(original_entity) else {
            warn!("Entity {:?} has no transform, skipping", original_entity);
            continue;
        };

        let transform = *transform;
        let name = world.get::<Name>(original_entity).map(|n| n.as_str().to_string());
        let mesh = world.get::<Mesh3d>(original_entity).cloned();
        let material = world.get::<MeshMaterial3d<StandardMaterial>>(original_entity).cloned();
        let is_group = world.get::<Group>(original_entity).is_some();
        let glb_model = world.get::<GlbModel>(original_entity).cloned();
        let has_scene_root = world.get::<SceneRoot>(original_entity).is_some();
        let rb_type = world.get::<RigidBodyType>(original_entity).copied();
        let visibility = world.get::<Visibility>(original_entity).copied();
        let is_player_spawn = world.get::<PlayerSpawn>(original_entity).is_some();
        let editor_light = world.get::<EditorLight>(original_entity).cloned();
        let point_light = world.get::<PointLight>(original_entity).cloned();
        let spot_light = world.get::<SpotLight>(original_entity).cloned();
        let children = world.get::<Children>(original_entity).map(|c| c.iter().collect::<Vec<_>>());

        entities_data.push((
            original_entity,
            transform,
            name,
            mesh,
            material,
            is_group,
            glb_model,
            has_scene_root,
            rb_type,
            visibility,
            is_player_spawn,
            editor_light,
            point_light,
            spot_light,
            children,
        ));
    }

    // First pass: Clone all entities
    for (
        original_entity,
        transform,
        name,
        mesh,
        material,
        is_group,
        glb_model,
        has_scene_root,
        rb_type,
        visibility,
        is_player_spawn,
        editor_light,
        point_light,
        spot_light,
        _children,
    ) in &entities_data {

        // Create new transform with offset
        let new_transform = Transform {
            translation: transform.translation + offset,
            rotation: transform.rotation,
            scale: transform.scale,
        };

        // Spawn new entity with base components
        let mut new_entity = world.spawn((
            EditorEntity,
            new_transform,
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

        // Clone Name component
        if let Some(name) = name {
            let new_name = format!("{} Copy", name);
            new_entity.insert(Name::new(new_name));
        } else {
            new_entity.insert(Name::new("Copy"));
        }

        // Clone Mesh3d
        if let Some(mesh) = mesh {
            new_entity.insert(mesh.clone());
        }

        // Clone Material
        if let Some(material) = material {
            new_entity.insert(material.clone());
        }

        // Clone Group marker
        if *is_group {
            new_entity.insert(Group);
        }

        // Clone GlbModel (SceneRoot handled after)
        if let Some(glb) = glb_model {
            new_entity.insert(glb.clone());
        }

        // Clone RigidBodyType
        if let Some(rb_type) = rb_type {
            new_entity.insert(*rb_type);
        }

        // Clone Visibility (override default if custom)
        if let Some(vis) = visibility {
            new_entity.insert(*vis);
        }

        // Clone PlayerSpawn marker
        if *is_player_spawn {
            new_entity.insert(PlayerSpawn);
        }

        // Clone light components
        if let Some(editor_light) = editor_light {
            new_entity.insert(editor_light.clone());

            if let Some(point_light) = point_light {
                new_entity.insert(point_light.clone());
            }

            if let Some(spot_light) = spot_light {
                new_entity.insert(spot_light.clone());
            }
        }

        let cloned_entity = new_entity.id();

        // Handle SceneRoot separately (requires asset_server)
        if *has_scene_root {
            if let Some(glb) = glb_model {
                let glb_path_str = glb.path.to_string_lossy().to_string();
                drop(new_entity); // Release the borrow

                let asset_server = world.resource::<AssetServer>();
                let scene_handle = asset_server.load(format!("{}#Scene0", glb_path_str));
                world.entity_mut(cloned_entity).insert(SceneRoot(scene_handle));
            }
        }

        entity_map.insert(*original_entity, cloned_entity);

        info!("Duplicated entity {:?} -> {:?}", original_entity, cloned_entity);
    }

    // Second pass: Clone children (like light visualization meshes)
    for (
        original_entity,
        _transform,
        _name,
        _mesh,
        _material,
        _is_group,
        _glb_model,
        _has_scene_root,
        _rb_type,
        _visibility,
        _is_player_spawn,
        _editor_light,
        _point_light,
        _spot_light,
        children,
    ) in &entities_data {
        let cloned_entity = entity_map[original_entity];

        if let Some(children_list) = children {
            for &child_entity in children_list {
                // Only clone EditorVisualization children
                if world.get::<EditorVisualization>(child_entity).is_some() {
                    // Collect child data
                    let child_transform = world.get::<Transform>(child_entity).copied();
                    let child_mesh = world.get::<Mesh3d>(child_entity).cloned();
                    let child_material = world.get::<MeshMaterial3d<StandardMaterial>>(child_entity).cloned();

                    let mut child_builder = world.spawn(EditorVisualization);

                    if let Some(transform) = child_transform {
                        child_builder.insert(transform);
                    }

                    if let Some(mesh) = child_mesh {
                        child_builder.insert(mesh);
                    }

                    if let Some(material) = child_material {
                        child_builder.insert(material);
                    }

                    let new_child = child_builder.id();
                    world.entity_mut(cloned_entity).add_child(new_child);
                    info!("Cloned visualization child {:?} -> {:?}", child_entity, new_child);
                }
            }
        }
    }

    // Third pass: Set up parent-child relationships
    for &original_entity in &selected_entities {
        let cloned_entity = entity_map[&original_entity];

        if let Some(child_of) = world.get::<ChildOf>(original_entity) {
            if let Some(&cloned_parent) = entity_map.get(&child_of.0) {
                world.entity_mut(cloned_entity).insert(ChildOf(cloned_parent));
                info!("Set parent of {:?} to {:?}", cloned_entity, cloned_parent);
            }
        }
    }

    // Clear previous selection and select duplicated entities
    for entity in selected_entities {
        world.entity_mut(entity).remove::<Selected>();
    }

    let mut selection = world.resource_mut::<SelectionSet>();
    selection.clear();
    drop(selection); // Release the borrow before inserting Selected components

    for &cloned_entity in entity_map.values() {
        world.resource_mut::<SelectionSet>().add(cloned_entity);
        world.entity_mut(cloned_entity).insert(Selected);
    }

    info!("Duplicated {} entities with +1m X-axis offset", entity_map.len());
}
