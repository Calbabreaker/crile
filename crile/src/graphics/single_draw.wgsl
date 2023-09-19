// Non-instanced (one draw call per mesh) shader

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct CameraUniform {
    view_projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) texture_coords: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_projection * vec4<f32>(position, 0.0, 1.0);
    out.texture_coords = texture_coords;
    out.color = vec4<f32>(f32(instance_index) / 10.0, 1.0, 1.0, 1.0);
    return out;
}

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
   return textureSample(texture, texture_sampler, in.texture_coords) * in.color;
}
