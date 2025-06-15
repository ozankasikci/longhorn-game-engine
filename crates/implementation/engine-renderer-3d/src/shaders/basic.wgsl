// Basic 3D shader for the standalone renderer

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // For now, just apply camera transform (we'll add model transform later)
    let world_position = vec4<f32>(in.position, 1.0);
    out.world_pos = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.color = in.color;
    
    // For now, use a simple normal pointing towards camera
    // In a real implementation, we'd pass normals as vertex attributes
    out.normal = normalize(in.position);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Return the interpolated vertex color
    return vec4<f32>(in.color, 1.0);
}