//! Object outline system for visual selection feedback
//!
//! This module provides visual highlighting for selected objects by rendering
//! a wireframe outline around them. The outline is implemented as a separate
//! entity parented to the target object, allowing it to follow transforms.
//!
//! # Architecture
//!
//! - **Outlined component**: Marker indicating an entity should have an outline
//! - **Outline entity**: Child entity with wireframe mesh matching parent shape
//! - **Automatic sync**: Outline transforms update to match parent
//!
//! # Lifecycle
//!
//! 1. Selection system adds `Outlined` component to selected entities
//! 2. `spawn_outlines()` observer creates outline mesh child
//! 3. `sync_outline_transforms()` keeps outline in sync with parent
//! 4. `despawn_outlines()` removes outline when `Outlined` component removed

use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

use crate::editor::core::materials::OutlineMaterial;

/// Marker component - add to entities to outline them
#[derive(Component)]
pub struct Outlined {
    pub size: f32, // Scale multiplier for outline (default: 1.05 = 5% larger)
}

impl Default for Outlined {
    fn default() -> Self {
        Self { size: 1.05 }
    }
}

/// Marker component for outline child entities that tracks the parent
#[derive(Component)]
pub struct OutlineMarker {
    pub parent: Entity,
}

/// Create an outline mesh by inverting normals
fn create_outline_mesh(original_mesh: &Mesh) -> Mesh {
    let mut outline_mesh = original_mesh.clone();

    // Invert normals so back-face culling works correctly
    if let Some(VertexAttributeValues::Float32x3(normals)) =
        outline_mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for normal in normals.iter_mut() {
            normal[0] = -normal[0];
            normal[1] = -normal[1];
            normal[2] = -normal[2];
        }
    }

    outline_mesh
}

/// Spawn outline for newly Outlined entities
pub fn spawn_outlines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    outline_material: Res<OutlineMaterial>,
    query: Query<(Entity, &Mesh3d, &Outlined), Added<Outlined>>,
) {
    for (entity, mesh_handle, outlined) in query.iter() {
        // Get outline scale from the Outlined component
        let scale = outlined.size;

        // Get the original mesh and create inverted version
        if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
            let outline_mesh = create_outline_mesh(original_mesh);
            let outline_mesh_handle = meshes.add(outline_mesh);

            // Spawn outline as child entity
            let outline_entity = commands.spawn((
                Mesh3d(outline_mesh_handle),
                MeshMaterial3d(outline_material.handle.clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(scale),
                },
                OutlineMarker { parent: entity },
                Name::new("Outline"),
            )).id();

            // Make it a child of the outlined entity
            commands.entity(entity).add_child(outline_entity);
        }
    }
}

/// Despawn outline when Outlined component is removed
pub fn despawn_outlines(
    mut commands: Commands,
    mut removed: RemovedComponents<Outlined>,
    outline_query: Query<(Entity, &OutlineMarker)>,
) {
    for entity in removed.read() {
        // Find and despawn all outline children for this entity
        for (outline_entity, marker) in outline_query.iter() {
            if marker.parent == entity {
                commands.entity(outline_entity).despawn();
            }
        }
    }
}

/// Sync outline transforms with parent entities every frame
pub fn sync_outline_transforms(
    parent_query: Query<(&Transform, &Outlined), Without<OutlineMarker>>,
    mut outline_query: Query<(&mut Transform, &OutlineMarker), Without<Outlined>>,
) {
    for (mut outline_transform, marker) in outline_query.iter_mut() {
        if let Ok((_parent_transform, outlined)) = parent_query.get(marker.parent) {
            // Match parent position and rotation, but use configured scale
            outline_transform.translation = Vec3::ZERO;
            outline_transform.rotation = Quat::IDENTITY;
            // Scale relative to parent using Outlined.size
            outline_transform.scale = Vec3::splat(outlined.size);
        }
    }
}
