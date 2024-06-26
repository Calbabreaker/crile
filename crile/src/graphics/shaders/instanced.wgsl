// Instanced (one draw call per mesh and texture) shader

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct DrawUniform {
    transform: mat4x4<f32>,
}

struct Instance {
    transform: mat4x4<f32>,
    color: vec4<f32>,
} 

@group(0) @binding(0)
var<uniform> draw: DrawUniform;

@group(2) @binding(0)
var<storage, read> instances: array<Instance>;

@vertex
fn vs_main(
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) texture_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var instance = instances[instance_index];

    var out: VertexOutput;
    out.position = draw.transform * instance.transform * vec4<f32>(position, 0.0, 1.0);
    out.texture_coords = texture_coords;
    out.color = instance.color * color;
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
