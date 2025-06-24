// Test-Driven Development for Phase 20.2: Mesh Import Implementation
//
// This test defines the expected behavior for mesh importing functionality

use std::path::Path;

#[test]
fn test_mesh_data_structure() {
    // Test 1: Verify mesh data structures exist and are correct
    use engine_mesh_import::{Material, MeshData, Vertex};

    let vertex = Vertex {
        position: [1.0, 2.0, 3.0],
        normal: [0.0, 1.0, 0.0],
        tex_coords: [0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
    };

    assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
    assert_eq!(vertex.normal, [0.0, 1.0, 0.0]);
    assert_eq!(vertex.tex_coords, [0.5, 0.5]);

    let mesh = MeshData {
        name: "TestMesh".to_string(),
        vertices: vec![vertex],
        indices: vec![0, 1, 2],
        material: Some(Material::default()),
    };

    assert_eq!(mesh.name, "TestMesh");
    assert_eq!(mesh.vertices.len(), 1);
    assert_eq!(mesh.indices.len(), 3);
}

#[tokio::test]
async fn test_obj_importer() {
    // Test 2: Verify OBJ importer can load basic OBJ files
    use engine_asset_import::{AssetImporter, ImportContext, ImportSettings};
    use engine_mesh_import::ObjImporter;

    let importer = ObjImporter::new();

    // Check supported extensions
    assert!(importer.supported_extensions().contains(&"obj"));
    assert!(importer.can_import(Path::new("model.obj")));
    assert!(!importer.can_import(Path::new("model.fbx")));

    // Test with sample OBJ content
    let test_obj = r#"
# Simple triangle
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vn 0.0 0.0 1.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
f 1/1/1 2/2/1 3/3/1
"#;

    let _context = ImportContext::new(ImportSettings::default());

    // Test parsing OBJ content
    let mesh_data = importer.parse_obj_content(test_obj).unwrap();
    assert_eq!(mesh_data.vertices.len(), 3);
    assert_eq!(mesh_data.indices.len(), 3);
}

#[tokio::test]
async fn test_gltf_importer() {
    // Test 3: Verify glTF importer basics
    use engine_asset_import::AssetImporter;
    use engine_mesh_import::GltfImporter;

    let importer = GltfImporter::new();

    // Check supported extensions
    assert!(importer.supported_extensions().contains(&"gltf"));
    assert!(importer.supported_extensions().contains(&"glb"));
    assert!(importer.can_import(Path::new("model.gltf")));
    assert!(importer.can_import(Path::new("model.glb")));
    assert!(!importer.can_import(Path::new("model.obj")));
}

#[tokio::test]
async fn test_fbx_importer() {
    // Test 4: Verify FBX importer basics
    use engine_asset_import::AssetImporter;
    use engine_mesh_import::FbxImporter;

    let importer = FbxImporter::new();

    // Check supported extensions
    assert!(importer.supported_extensions().contains(&"fbx"));
    assert!(importer.can_import(Path::new("model.fbx")));
    assert!(!importer.can_import(Path::new("model.obj")));
}

#[test]
fn test_mesh_converter() {
    // Test 5: Verify mesh converter transforms data to engine format
    use engine_mesh_import::{MeshConverter, MeshData, Vertex};

    let mesh_data = MeshData {
        name: "TestMesh".to_string(),
        vertices: vec![
            Vertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [0.5, 1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.5, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ],
        indices: vec![0, 1, 2],
        material: None,
    };

    let converter = MeshConverter::new();
    let engine_mesh = converter.convert(&mesh_data).unwrap();

    // Verify conversion
    assert_eq!(engine_mesh.vertex_count(), 3);
    assert_eq!(engine_mesh.index_count(), 3);
}

#[test]
fn test_material_extraction() {
    // Test 6: Verify material data is extracted correctly
    use engine_mesh_import::{Material, MaterialProperty};

    let mut material = Material::new("TestMaterial");
    material.set_property(MaterialProperty::BaseColor([1.0, 0.0, 0.0, 1.0]));
    material.set_property(MaterialProperty::Metallic(0.5));
    material.set_property(MaterialProperty::Roughness(0.3));

    assert_eq!(material.name(), "TestMaterial");

    match material.get_property("base_color") {
        Some(MaterialProperty::BaseColor(color)) => {
            assert_eq!(color, &[1.0, 0.0, 0.0, 1.0]);
        }
        _ => panic!("Expected base color property"),
    }
}

#[test]
fn test_mesh_validation() {
    // Test 7: Verify mesh validation catches issues
    use engine_mesh_import::{MeshData, MeshValidator, ValidationError, Vertex};

    let validator = MeshValidator::new();

    // Test empty mesh
    let empty_mesh = MeshData {
        name: "Empty".to_string(),
        vertices: vec![],
        indices: vec![],
        material: None,
    };

    let result = validator.validate(&empty_mesh);
    assert!(result.is_err());
    assert!(matches!(result, Err(ValidationError::NoVertices)));

    // Test invalid indices
    let invalid_mesh = MeshData {
        name: "Invalid".to_string(),
        vertices: vec![Vertex::default(); 3],
        indices: vec![0, 1, 5], // Index 5 is out of bounds
        material: None,
    };

    let result = validator.validate(&invalid_mesh);
    assert!(result.is_err());
    assert!(matches!(result, Err(ValidationError::InvalidIndex(_, _))));
}

#[test]
fn test_mesh_optimization() {
    // Test 8: Verify mesh optimization works
    use engine_mesh_import::{MeshData, MeshOptimizer, Vertex};

    // Create mesh with duplicate vertices
    let mesh_data = MeshData {
        name: "Unoptimized".to_string(),
        vertices: vec![
            Vertex {
                position: [0.0, 0.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                ..Default::default()
            }, // Duplicate
            Vertex {
                position: [1.0, 1.0, 0.0],
                ..Default::default()
            },
        ],
        indices: vec![0, 1, 2, 2, 1, 3], // Using duplicate vertex
        material: None,
    };

    let optimizer = MeshOptimizer::new();
    let optimized = optimizer.optimize(mesh_data).unwrap();

    // Should have removed duplicate vertex
    assert_eq!(optimized.vertices.len(), 3);
    assert_eq!(optimized.indices, vec![0, 1, 0, 0, 1, 2]); // Remapped indices
}

#[test]
fn test_normal_generation() {
    // Test 9: Verify normal generation for meshes without normals
    use engine_mesh_import::{MeshData, NormalGenerator, Vertex};

    let mut mesh_data = MeshData {
        name: "NoNormals".to_string(),
        vertices: vec![
            Vertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: [0.5, 1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
                ..Default::default()
            },
        ],
        indices: vec![0, 1, 2],
        material: None,
    };

    let generator = NormalGenerator::new();
    generator.generate_normals(&mut mesh_data);

    // All vertices should have the same normal (facing +Z)
    for vertex in &mesh_data.vertices {
        assert!(vertex.normal[2] > 0.9); // Should be close to [0, 0, 1]
    }
}

#[test]
fn test_mesh_import_registry() {
    // Test 10: Verify mesh import registry manages all importers
    use engine_mesh_import::MeshImportRegistry;

    let registry = MeshImportRegistry::new();

    // Should have default importers registered
    // Note: get_importer returns different types for different formats
    // so we just check supported formats
    let formats = registry.supported_formats();
    assert!(formats.contains(&"obj"));
    assert!(formats.contains(&"gltf"));
    assert!(formats.contains(&"glb"));
    assert!(formats.contains(&"fbx"));

    // Should list all supported formats
    let formats = registry.supported_formats();
    assert!(formats.contains(&"obj"));
    assert!(formats.contains(&"gltf"));
    assert!(formats.contains(&"fbx"));
}

#[tokio::test]
async fn test_mesh_import_integration() {
    // Test 11: Integration test with asset import pipeline
    use engine_mesh_import::create_mesh_import_pipeline;

    let pipeline = create_mesh_import_pipeline();

    // Verify pipeline has mesh importers registered
    assert!(pipeline.find_importer(Path::new("test.obj")).is_some());
    assert!(pipeline.find_importer(Path::new("test.gltf")).is_some());
    assert!(pipeline.find_importer(Path::new("test.fbx")).is_some());
}

#[test]
fn test_mesh_bounds_calculation() {
    // Test 12: Verify bounding box calculation
    use engine_mesh_import::{calculate_bounds, MeshData, Vertex};

    let mesh_data = MeshData {
        name: "BoundsTest".to_string(),
        vertices: vec![
            Vertex {
                position: [-1.0, -1.0, -1.0],
                ..Default::default()
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                ..Default::default()
            },
            Vertex {
                position: [0.0, 2.0, -2.0],
                ..Default::default()
            },
        ],
        indices: vec![0, 1, 2],
        material: None,
    };

    let bounds = calculate_bounds(&mesh_data);
    assert_eq!(bounds.min, [-1.0, -1.0, -2.0]);
    assert_eq!(bounds.max, [1.0, 2.0, 1.0]);
}
