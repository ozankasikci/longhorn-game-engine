// Scene shader for rendering 3D objects in the editor

struct SceneUniform {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> scene_uniform: SceneUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space
    let world_pos = scene_uniform.model * vec4<f32>(input.position, 1.0);
    out.world_pos = world_pos.xyz;
    
    // Transform normal to world space
    let normal_matrix = mat3x3<f32>(
        scene_uniform.model[0].xyz,
        scene_uniform.model[1].xyz,
        scene_uniform.model[2].xyz
    );
    out.world_normal = normalize(normal_matrix * input.normal);
    
    // Pass through texture coordinates
    out.tex_coords = input.tex_coords;
    
    // Calculate final clip position
    out.clip_position = scene_uniform.view_proj * world_pos;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(0.5, -1.0, -0.3));
    let ambient = 0.2;
    let diffuse = max(dot(in.world_normal, -light_dir), 0.0);
    
    // Basic material color (will be replaced with actual material data later)
    let base_color = vec3<f32>(0.8, 0.8, 0.8);
    
    // Calculate final color
    let lit_color = base_color * (ambient + diffuse);
    
    return vec4<f32>(lit_color, 1.0);
}