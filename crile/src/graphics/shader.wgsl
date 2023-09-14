struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texture_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) texture_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texture_index: u32,
};

struct CameraUniform {
    view_projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_projection * vec4<f32>(vertex.position, 0.0, 1.0);
    out.texture_coords = vertex.texture_coords;
    out.color = vertex.color;
    out.texture_index = vertex.texture_index;
    return out;
}

@group(1) @binding(0)
var texture_array: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var sampler_array: binding_array<sampler>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
   return textureSample(texture_array[in.texture_index], sampler_array[in.texture_index], in.texture_coords);
}
