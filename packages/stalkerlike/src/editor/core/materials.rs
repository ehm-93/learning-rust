use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

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
