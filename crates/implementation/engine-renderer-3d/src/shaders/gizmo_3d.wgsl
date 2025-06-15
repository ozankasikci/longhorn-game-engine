// 3D Gizmo shader with constant screen-space sizing

struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    gizmo_position: vec4<f32>,
    camera_position: vec4<f32>,
    viewport_size: vec4<f32>, // x: width, y: height, z: gizmo_size, w: unused
    highlight_color: vec4<f32>,
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
    
    // Calculate view space position of gizmo center
    let gizmo_view_pos = uniforms.view * uniforms.gizmo_position;
    let distance = length(gizmo_view_pos.xyz);
    
    // Calculate scale factor for constant screen size
    // The gizmo should appear as a fixed pixel size on screen
    let fov_y = 1.0472; // 60 degrees in radians (should be passed as uniform in production)
    let scale_factor = (uniforms.viewport_size.z / uniforms.viewport_size.y) * distance * tan(fov_y * 0.5) * 2.0;
    
    // Scale the vertex position
    var scaled_position = in.position * scale_factor;
    
    // Apply axis-specific rotation based on vertex color
    // Red (X-axis): rotate 90 degrees around Z
    // Green (Y-axis): no rotation (already pointing up)
    // Blue (Z-axis): rotate 90 degrees around X
    if (in.color.r > 0.8 && in.color.g < 0.4 && in.color.b < 0.4) {
        // Red X-axis: rotate to point along X
        let temp = scaled_position.x;
        scaled_position.x = scaled_position.y;
        scaled_position.y = -temp;
    } else if (in.color.b > 0.8 && in.color.r < 0.4 && in.color.g < 0.4) {
        // Blue Z-axis: rotate to point along Z
        let temp = scaled_position.y;
        scaled_position.y = scaled_position.z;
        scaled_position.z = temp;
    }
    // Green Y-axis needs no rotation (already correct)
    
    // Add scaled position to gizmo center (don't apply object's rotation/scale to gizmo)
    let world_position = uniforms.gizmo_position + vec4<f32>(scaled_position, 0.0);
    out.world_pos = world_position.xyz;
    
    // Transform to clip space
    out.clip_position = uniforms.projection * uniforms.view * world_position;
    
    // Pass through color
    out.color = in.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Basic shading with highlight support
    var color = in.color;
    
    // Add slight shading based on view direction for better depth perception
    let view_dir = normalize(uniforms.camera_position.xyz - in.world_pos);
    let shading = 0.8 + 0.2 * max(0.0, view_dir.z);
    
    color = vec4<f32>(color.rgb * shading, color.a);
    
    return color;
}