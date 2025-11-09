#import bevy_pbr::forward_io::VertexOutput
#import bevy_render::view::View

struct GridMaterial {
    color: vec4<f32>,
    fade_start: f32,
    fade_end: f32,
}

@group(2) @binding(0)
var<uniform> material: GridMaterial;

@group(0) @binding(0)
var<uniform> view: View;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // Calculate distance from camera to fragment in world space
    let world_position = mesh.world_position.xyz;
    let camera_position = view.world_position;
    let distance = length(camera_position - world_position);

    // Calculate fade based on distance
    // fade_factor goes from 1.0 (close) to 0.0 (far)
    let fade_factor = 1.0 - smoothstep(material.fade_start, material.fade_end, distance);

    // Apply fade to alpha channel
    let alpha = material.color.a * fade_factor;

    return vec4<f32>(material.color.rgb, alpha);
}
