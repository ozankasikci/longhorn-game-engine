//! Integration tests for engine-materials-core

use engine_materials_core::*;
use glam::Vec3;

#[test]
fn test_color_creation_and_conversions() {
    // Test basic color creation
    let white = Color::WHITE;
    assert_eq!(white.r, 1.0);
    assert_eq!(white.g, 1.0);
    assert_eq!(white.b, 1.0);
    assert_eq!(white.a, 1.0);

    let red = Color::RED;
    assert_eq!(red.r, 1.0);
    assert_eq!(red.g, 0.0);
    assert_eq!(red.b, 0.0);

    // Test color creation from different sources
    let from_rgb = Color::rgb(0.5, 0.7, 0.9);
    assert_eq!(from_rgb.r, 0.5);
    assert_eq!(from_rgb.g, 0.7);
    assert_eq!(from_rgb.b, 0.9);
    assert_eq!(from_rgb.a, 1.0);

    let from_hex = Color::hex(0xFF8000); // Orange
    assert!((from_hex.r - 1.0).abs() < 0.01);
    assert!((from_hex.g - 0.5).abs() < 0.01);
    assert!((from_hex.b - 0.0).abs() < 0.01);

    // Test color space conversions
    let linear = Color::rgb(0.5, 0.5, 0.5);
    let srgb = linear.to_srgb();
    let back_to_linear = srgb.to_linear();
    
    // Should be close to original after round-trip conversion
    assert!((linear.r - back_to_linear.r).abs() < 0.01);
    assert!((linear.g - back_to_linear.g).abs() < 0.01);
    assert!((linear.b - back_to_linear.b).abs() < 0.01);
}

#[test]
fn test_color_operations() {
    let red = Color::RED;
    let blue = Color::BLUE;

    // Test multiplication
    let purple = red.multiply(&blue);
    assert_eq!(purple.r, 0.0); // Red * Blue = (1,0,0) * (0,0,1) = (0,0,0) in RGB
    assert_eq!(purple.g, 0.0);
    assert_eq!(purple.b, 0.0);

    // Test addition
    let magenta = red.add(&blue);
    assert_eq!(magenta.r, 1.0);
    assert_eq!(magenta.g, 0.0);
    assert_eq!(magenta.b, 1.0);

    // Test scaling
    let dark_red = red.scale(0.5);
    assert_eq!(dark_red.r, 0.5);
    assert_eq!(dark_red.g, 0.0);
    assert_eq!(dark_red.b, 0.0);

    // Test interpolation
    let lerp_color = red.lerp(&blue, 0.5);
    assert_eq!(lerp_color.r, 0.5);
    assert_eq!(lerp_color.g, 0.0);
    assert_eq!(lerp_color.b, 0.5);

    // Test luminance
    let white_luminance = Color::WHITE.luminance();
    let black_luminance = Color::BLACK.luminance();
    assert!(white_luminance > black_luminance);
    assert!((white_luminance - 1.0).abs() < 0.01);
    assert!((black_luminance - 0.0).abs() < 0.01);
}

#[test]
fn test_color_conversions() {
    let color = Color::rgb(0.8, 0.6, 0.4);

    // Test array conversions
    let array = color.to_array();
    assert_eq!(array, [0.8, 0.6, 0.4, 1.0]);

    let rgb_array = color.to_rgb_array();
    assert_eq!(rgb_array, [0.8, 0.6, 0.4]);

    // Test Vec3 conversion
    let vec3 = color.to_vec3();
    assert_eq!(vec3, Vec3::new(0.8, 0.6, 0.4));

    // Test From trait implementations
    let from_array: Color = [0.1, 0.2, 0.3, 0.4].into();
    assert_eq!(from_array.r, 0.1);
    assert_eq!(from_array.g, 0.2);
    assert_eq!(from_array.b, 0.3);
    assert_eq!(from_array.a, 0.4);

    let from_rgb_array: Color = [0.5, 0.6, 0.7].into();
    assert_eq!(from_rgb_array.r, 0.5);
    assert_eq!(from_rgb_array.g, 0.6);
    assert_eq!(from_rgb_array.b, 0.7);
    assert_eq!(from_rgb_array.a, 1.0);

    let from_vec3: Color = Vec3::new(0.9, 0.8, 0.7).into();
    assert_eq!(from_vec3.r, 0.9);
    assert_eq!(from_vec3.g, 0.8);
    assert_eq!(from_vec3.b, 0.7);
    assert_eq!(from_vec3.a, 1.0);
}

#[test]
fn test_hsl_color_space() {
    // Test HSL creation
    let red_hsl = Color::hsl(0.0, 1.0, 0.5); // Pure red
    assert!((red_hsl.r - 1.0).abs() < 0.01);
    assert!(red_hsl.g.abs() < 0.01);
    assert!(red_hsl.b.abs() < 0.01);

    let green_hsl = Color::hsl(120.0, 1.0, 0.5); // Pure green
    assert!(green_hsl.r.abs() < 0.01);
    assert!((green_hsl.g - 1.0).abs() < 0.01);
    assert!(green_hsl.b.abs() < 0.01);

    // Test HSL conversion
    let white = Color::WHITE;
    let (_h, s, l) = white.to_hsl();
    assert_eq!(s, 0.0); // White has no saturation
    assert_eq!(l, 1.0); // White has maximum lightness

    let black = Color::BLACK;
    let (_h, _s, l) = black.to_hsl();
    assert_eq!(l, 0.0); // Black has zero lightness
}

#[test]
fn test_material_creation_and_properties() {
    // Test default material
    let default_material = Material::default();
    assert_eq!(default_material.name, "Default");
    assert_eq!(default_material.pbr.albedo, Color::WHITE);
    assert_eq!(default_material.pbr.metallic, 0.0);
    assert_eq!(default_material.pbr.roughness, 0.5);
    assert_eq!(default_material.alpha_mode, AlphaMode::Opaque);
    assert!(!default_material.double_sided);
    assert!(!default_material.unlit);

    // Test custom material creation
    let mut custom_material = Material::default();
    custom_material.name = "Metal".to_string();
    custom_material.pbr.albedo = Color::rgb(0.7, 0.7, 0.8);
    custom_material.pbr.metallic = 1.0;
    custom_material.pbr.roughness = 0.1;
    custom_material.double_sided = true;

    assert_eq!(custom_material.name, "Metal");
    assert_eq!(custom_material.pbr.metallic, 1.0);
    assert_eq!(custom_material.pbr.roughness, 0.1);
    assert!(custom_material.double_sided);
}

#[test]
fn test_alpha_modes() {
    // Test alpha mode variants
    let opaque = AlphaMode::Opaque;
    let mask = AlphaMode::Mask { cutoff: 0.5 };
    let blend = AlphaMode::Blend;

    // Ensure they can be compared (PartialEq)
    assert_eq!(opaque, AlphaMode::Opaque);
    assert_ne!(opaque, blend);
    
    // Test pattern matching
    match mask {
        AlphaMode::Mask { cutoff } => {
            assert_eq!(cutoff, 0.5);
        }
        _ => panic!("Expected Mask variant"),
    }
}

#[test]
fn test_shader_creation_and_properties() {
    // Test shader creation from text
    let vertex_shader = Shader::from_text(
        "basic_vertex",
        ShaderType::Vertex,
        ShaderLanguage::Wgsl,
        "
        @vertex
        fn main() -> @builtin(position) vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
        "
    );

    assert_eq!(vertex_shader.name, "basic_vertex");
    assert_eq!(vertex_shader.shader_type, ShaderType::Vertex);
    assert_eq!(vertex_shader.language, ShaderLanguage::Wgsl);
    assert_eq!(vertex_shader.entry_point, "vs_main");
    assert!(vertex_shader.source_text().is_some());
    assert!(vertex_shader.source_binary().is_none());

    // Test shader creation from binary
    let binary_data = vec![0x03, 0x02, 0x23, 0x07]; // SPIR-V magic number
    let spirv_shader = Shader::from_binary(
        "compiled_shader",
        ShaderType::Fragment,
        ShaderLanguage::SpirV,
        binary_data.clone()
    );

    assert_eq!(spirv_shader.name, "compiled_shader");
    assert_eq!(spirv_shader.shader_type, ShaderType::Fragment);
    assert_eq!(spirv_shader.language, ShaderLanguage::SpirV);
    assert!(spirv_shader.source_text().is_none());
    assert_eq!(spirv_shader.source_binary(), Some(binary_data.as_slice()));

    // Test custom entry point
    let custom_shader = Shader::from_text(
        "custom",
        ShaderType::Compute,
        ShaderLanguage::Hlsl,
        "void MyCustomMain() {}"
    ).with_entry_point("MyCustomMain");

    assert_eq!(custom_shader.entry_point, "MyCustomMain");
}

#[test]
fn test_shader_program() {
    let mut program = ShaderProgram::new("basic_program");
    
    assert_eq!(program.name, "basic_program");
    assert!(program.shaders.is_empty());
    assert!(!program.is_graphics_pipeline());
    assert!(!program.is_compute_pipeline());

    // Add shaders
    program.add_shader(1, ShaderType::Vertex);
    program.add_shader(2, ShaderType::Fragment);

    assert_eq!(program.shaders.len(), 2);
    assert_eq!(program.vertex_shader, Some(1));
    assert_eq!(program.fragment_shader, Some(2));
    assert!(program.is_graphics_pipeline());
    assert!(!program.is_compute_pipeline());

    // Test compute pipeline
    let mut compute_program = ShaderProgram::new("compute_program");
    compute_program.add_shader(3, ShaderType::Compute);

    assert!(!compute_program.is_graphics_pipeline());
    assert!(compute_program.is_compute_pipeline());
}

#[test]
fn test_vertex_format_properties() {
    // Test vertex format sizes
    assert_eq!(VertexFormat::Float32.size(), 4);
    assert_eq!(VertexFormat::Float32x2.size(), 8);
    assert_eq!(VertexFormat::Float32x3.size(), 12);
    assert_eq!(VertexFormat::Float32x4.size(), 16);
    assert_eq!(VertexFormat::Uint16x2.size(), 4);
    assert_eq!(VertexFormat::Uint8x4.size(), 4);

    // Test component counts
    assert_eq!(VertexFormat::Float32.component_count(), 1);
    assert_eq!(VertexFormat::Float32x2.component_count(), 2);
    assert_eq!(VertexFormat::Float32x3.component_count(), 3);
    assert_eq!(VertexFormat::Float32x4.component_count(), 4);
    assert_eq!(VertexFormat::Uint16x2.component_count(), 2);
    assert_eq!(VertexFormat::Uint8x4.component_count(), 4);
}

#[test]
fn test_material_textures() {
    let mut textures = MaterialTextures::default();
    
    // Initially all textures should be None
    assert!(textures.albedo.is_none());
    assert!(textures.metallic_roughness.is_none());
    assert!(textures.normal.is_none());
    assert!(textures.emission.is_none());
    assert!(textures.occlusion.is_none());

    // Add some texture handles
    textures.albedo = Some(1);
    textures.normal = Some(2);
    textures.metallic_roughness = Some(3);

    assert_eq!(textures.albedo, Some(1));
    assert_eq!(textures.normal, Some(2));
    assert_eq!(textures.metallic_roughness, Some(3));
    assert!(textures.emission.is_none());
    assert!(textures.occlusion.is_none());
}