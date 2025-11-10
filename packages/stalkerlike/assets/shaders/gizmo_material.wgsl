#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
}

struct GizmoMaterial {
    color_r: f32,
    color_g: f32,
    color_b: f32,
    color_a: f32,
    emissive_r: f32,
    emissive_g: f32,
    emissive_b: f32,
    emissive_a: f32,
}

@group(2) @binding(0)
var<uniform> material: GizmoMaterial;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // Simple unlit shader that combines base color and emissive
    let color = vec4<f32>(material.color_r, material.color_g, material.color_b, material.color_a);
    let emissive = vec4<f32>(material.emissive_r, material.emissive_g, material.emissive_b, material.emissive_a);
    return color + emissive;
}
