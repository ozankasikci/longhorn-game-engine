// Basic 3D shader for the standalone renderer

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

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
    // Apply model transform, then camera transform
    let world_position = model.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.color = in.color;
    
    // Transform normal by model matrix (without translation)
    // In a real implementation, we'd use the normal matrix (inverse transpose of model)
    out.normal = normalize((model.model * vec4<f32>(in.position, 0.0)).xyz);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Return the interpolated vertex color
    return vec4<f32>(in.color, 1.0);
}