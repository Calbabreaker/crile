// Non-instanced (one draw call per mesh) shader

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct DrawUniform {
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> draw: DrawUniform;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) texture_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = draw.transform * vec4<f32>(position, 0.0, 1.0);
    out.texture_coords = texture_coords;
    out.color = color;
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
