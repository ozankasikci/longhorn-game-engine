//! Integration tests for MeshFilter and MeshRenderer components with modern ECS
//! These tests verify that the new mesh components work properly

use engine_components_3d::{MeshFilter, MeshRenderer};
use engine_resource_core::{ResourceId, ResourceHandle, Resource};
use engine_geometry_core::{MeshData, Vertex};
use glam::{Vec3, Vec2};

fn create_test_mesh_data() -> MeshData {
    let vertices = vec![
        Vertex {
            position: Vec3::new(-1.0, -1.0, 0.0),
            normal: Vec3::Z,
            uv: Vec2::new(0.0, 0.0),
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: Vec3::new(1.0, -1.0, 0.0),
            normal: Vec3::Z,
            uv: Vec2::new(1.0, 0.0),
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: Vec3::new(0.0, 1.0, 0.0),
            normal: Vec3::Z,
            uv: Vec2::new(0.5, 1.0),
            color: [1.0, 1.0, 1.0, 1.0],
        },
    ];
    
    MeshData::new(
        "test_triangle".to_string(),
        vertices,
        vec![0, 1, 2],
    )
}

#[test]
fn test_mesh_data_creation() {
    let mesh_data = create_test_mesh_data();
    
    assert_eq!(mesh_data.name, "test_triangle");
    assert_eq!(mesh_data.vertex_count(), 3);
    assert_eq!(mesh_data.index_count(), 3);
    assert_eq!(mesh_data.submeshes.len(), 1);
    assert_eq!(mesh_data.submeshes[0].start_index, 0);
    assert_eq!(mesh_data.submeshes[0].index_count, 3);
    assert_eq!(mesh_data.submeshes[0].material_index, 0);
    assert!(mesh_data.bounds.is_valid());
}

#[test]
fn test_mesh_filter_functionality() {
    let mesh_handle = ResourceHandle::<MeshData>::new(ResourceId::new(100));
    let mesh_filter = MeshFilter::new(mesh_handle.clone());
    
    assert_eq!(mesh_filter.mesh.id(), mesh_handle.id());
}

#[test]
fn test_mesh_renderer_functionality() {
    // Test default creation
    let renderer1 = MeshRenderer::default();
    assert_eq!(renderer1.materials.len(), 1);
    assert_eq!(renderer1.materials[0], 0);
    assert!(renderer1.cast_shadows);
    assert!(renderer1.receive_shadows);
    assert_eq!(renderer1.layer_mask, 0xFFFFFFFF);
    assert!(renderer1.enabled);
    
    // Test with single material
    let renderer2 = MeshRenderer::new(42);
    assert_eq!(renderer2.materials[0], 42);
    
    // Test with multiple materials
    let materials = vec![10, 20, 30];
    let renderer3 = MeshRenderer::with_materials(materials.clone());
    assert_eq!(renderer3.materials, materials);
    
    // Test builder pattern
    let renderer4 = MeshRenderer::new(5)
        .with_shadows(false, true)
        .with_layer_mask(0xFF00FF00);
    assert!(!renderer4.cast_shadows);
    assert!(renderer4.receive_shadows);
    assert_eq!(renderer4.layer_mask, 0xFF00FF00);
}

#[test]
fn test_mesh_data_bounds() {
    let vertices = vec![
        Vertex {
            position: Vec3::new(-2.0, -3.0, -4.0),
            normal: Vec3::Y,
            uv: Vec2::ZERO,
            color: [1.0; 4],
        },
        Vertex {
            position: Vec3::new(5.0, 6.0, 7.0),
            normal: Vec3::Y,
            uv: Vec2::ONE,
            color: [1.0; 4],
        },
    ];
    
    let mesh_data = MeshData::new(
        "bounds_test".to_string(),
        vertices,
        vec![0, 1],
    );
    
    assert_eq!(mesh_data.bounds.min, Vec3::new(-2.0, -3.0, -4.0));
    assert_eq!(mesh_data.bounds.max, Vec3::new(5.0, 6.0, 7.0));
}

#[test]
fn test_mesh_handle_comparison() {
    let handle1 = ResourceHandle::<MeshData>::new(ResourceId::new(100));
    let handle2 = ResourceHandle::<MeshData>::new(ResourceId::new(100));
    let handle3 = ResourceHandle::<MeshData>::new(ResourceId::new(200));
    
    assert!(handle1.same_resource(&handle2));
    assert!(!handle1.same_resource(&handle3));
}

#[test]
fn test_mesh_data_memory_usage() {
    let mesh_data = create_test_mesh_data();
    
    let expected_vertex_size = 3 * std::mem::size_of::<Vertex>();
    let expected_index_size = 3 * std::mem::size_of::<u32>();
    let expected_submesh_size = 1 * std::mem::size_of::<engine_geometry_core::SubMesh>();
    let expected_total = expected_vertex_size + expected_index_size + expected_submesh_size;
    
    assert_eq!(mesh_data.memory_usage(), expected_total);
    assert_eq!(mesh_data.memory_size(), expected_total);
}

#[test]
fn test_mesh_data_with_multiple_submeshes() {
    let vertices = create_test_mesh_data().vertices;
    let indices = vec![0, 1, 2, 0, 2, 1]; // Two triangles
    
    let mut mesh_data = MeshData::new(
        "multi_submesh".to_string(),
        vertices,
        indices,
    );
    
    // Define two submeshes
    mesh_data.submeshes = vec![
        engine_geometry_core::SubMesh {
            start_index: 0,
            index_count: 3,
            material_index: 0,
        },
        engine_geometry_core::SubMesh {
            start_index: 3,
            index_count: 3,
            material_index: 1,
        },
    ];
    
    assert_eq!(mesh_data.submeshes.len(), 2);
    assert_eq!(mesh_data.submeshes[0].material_index, 0);
    assert_eq!(mesh_data.submeshes[1].material_index, 1);
}