// Grid shader for rendering reference grid in 3D scenes

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fade_params: vec4<f32>, // x: fade_enabled, y: max_distance
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_pos: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    out.color = in.color;
    out.world_pos = in.position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    
    // Apply distance fade if enabled
    if (uniforms.fade_params.x > 0.5) {
        let distance = length(in.world_pos - uniforms.camera_pos.xyz);
        let fade_start = uniforms.fade_params.y * 0.5;
        let fade_end = uniforms.fade_params.y;
        
        // Calculate fade factor
        let fade = 1.0 - smoothstep(fade_start, fade_end, distance);
        color.a *= fade;
    }
    
    return color;
}