//! Custom shader materials for editor visualization
//!
//! This module defines custom materials used by the editor for various visual effects:
//!
//! - **GridMaterial**: Infinite grid with distance-based fade
//! - **GizmoMaterial**: Colored materials for transform gizmo handles
//! - **OutlineMaterial**: Material for selection outlines
//!
//! These materials use custom shaders to achieve effects not possible with
//! standard PBR materials.

use bevy::prelude::*;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::render::render_resource::{
    AsBindGroup, ShaderRef, RenderPipelineDescriptor, CompareFunction,
    SpecializedMeshPipelineError, Face,
};

/// Custom material for grid lines with distance-based fade
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub fade_start: f32,
    #[uniform(0)]
    pub fade_end: f32,
}

impl Material for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Custom material for gizmo that always renders on top
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GizmoMaterial {
    #[uniform(0)]
    pub color_r: f32,
    #[uniform(0)]
    pub color_b: f32,
    #[uniform(0)]
    pub color_g: f32,
    #[uniform(0)]
    pub color_a: f32,
    #[uniform(0)]
    pub emissive_r: f32,
    #[uniform(0)]
    pub emissive_g: f32,
    #[uniform(0)]
    pub emissive_b: f32,
    #[uniform(0)]
    pub emissive_a: f32,
}

impl Material for GizmoMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gizmo_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Disable depth testing so gizmo always renders on top
        if let Some(depth_stencil) = &mut descriptor.depth_stencil {
            depth_stencil.depth_compare = CompareFunction::Always;
            depth_stencil.depth_write_enabled = false;
        }
        Ok(())
    }
}

impl GizmoMaterial {
    /// Create a new gizmo material with given colors
    pub fn new(color: LinearRgba, emissive: LinearRgba) -> Self {
        Self {
            color_r: color.red,
            color_g: color.green,
            color_b: color.blue,
            color_a: color.alpha,
            emissive_r: emissive.red,
            emissive_g: emissive.green,
            emissive_b: emissive.blue,
            emissive_a: emissive.alpha,
        }
    }
}

/// Resource holding the outline material handle
#[derive(Resource)]
pub struct OutlineMaterial {
    pub handle: Handle<StandardMaterial>,
}

impl FromWorld for OutlineMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let handle = materials.add(StandardMaterial {
            base_color: Color::srgba(0.95, 0.95, 0.9, 1.0), // Off-white
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            cull_mode: Some(Face::Front), // Cull front faces so only back faces show
            ..default()
        });

        OutlineMaterial { handle }
    }
}
