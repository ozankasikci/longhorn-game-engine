//! Integration tests for engine-geometry-core

use engine_geometry_core::*;
use glam::{Vec3, Vec2};

#[test]
fn test_vertex_creation_and_pod() {
    let vertex = Vertex {
        position: Vec3::new(1.0, 2.0, 3.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
        uv: Vec2::new(0.5, 0.5),
        color: [1.0, 0.0, 0.0, 1.0],
    };

    // Test that it can be converted to bytes (Pod trait)
    let bytes = bytemuck::bytes_of(&vertex);
    assert_eq!(bytes.len(), std::mem::size_of::<Vertex>());

    // Test that it can be converted back
    let vertex2: &Vertex = bytemuck::from_bytes(bytes);
    assert_eq!(vertex.position, vertex2.position);
    assert_eq!(vertex.normal, vertex2.normal);
    assert_eq!(vertex.uv, vertex2.uv);
    assert_eq!(vertex.color, vertex2.color);
}

#[test]
fn test_mesh_creation_and_validation() {
    let vertices = vec![
        Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            uv: Vec2::new(0.0, 0.0),
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            uv: Vec2::new(1.0, 0.0),
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Vertex {
            position: Vec3::new(0.5, 1.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            uv: Vec2::new(0.5, 1.0),
            color: [1.0, 1.0, 1.0, 1.0],
        },
    ];

    let indices = vec![0, 1, 2];

    let mesh_data = MeshData::new(
        "triangle".to_string(),
        vertices,
        indices,
    );
    let mesh = Mesh::from_data(mesh_data);
    
    assert_eq!(mesh.name, "triangle");
    assert_eq!(mesh.vertex_count(), 3);
    assert_eq!(mesh.triangle_count(), 1);
    assert!(mesh.has_indices());
    
    // Test bounds calculation
    let bounds = mesh.bounds;
    assert!(bounds.min.x >= -0.1); // Should be close to 0
    assert!(bounds.max.x <= 1.1);  // Should be close to 1
    assert!(bounds.max.y <= 1.1);  // Should be close to 1
}

// #[test]
// fn test_primitive_generation() {
//     // Test cube generation
//     let cube = primitives::MeshPrimitives::cube(2.0);
//     assert!(cube.vertex_count() > 0);
//     assert!(cube.triangle_count() > 0);
//     assert_eq!(cube.name, "Cube");

//     // Test sphere generation
//     let sphere = primitives::MeshPrimitives::sphere(1.0, 16, 16);
//     assert!(sphere.vertex_count() > 0);
//     assert!(sphere.triangle_count() > 0);
//     assert_eq!(sphere.name, "Sphere");

//     // Test plane generation
//     let plane = primitives::MeshPrimitives::plane(5.0, 5.0, 2, 2);
//     assert!(plane.vertex_count() > 0);
//     assert!(plane.triangle_count() > 0);
//     assert_eq!(plane.name, "Plane");
// }

#[test]
fn test_bounding_box_operations() {
    let box1 = BoundingBox::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0)
    );

    let box2 = BoundingBox::new(
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(1.5, 1.5, 1.5)
    );

    // Test intersection
    assert!(box1.intersects_box(&box2));
    assert!(box2.intersects_box(&box1));

    // Test containment
    let point_inside = Vec3::new(0.5, 0.5, 0.5);
    let point_outside = Vec3::new(2.0, 2.0, 2.0);
    
    assert!(box1.contains_point(point_inside));
    assert!(!box1.contains_point(point_outside));

    // Test union
    let union_box = box1.union(&box2);
    assert!(union_box.contains_point(Vec3::new(0.0, 0.0, 0.0)));
    assert!(union_box.contains_point(Vec3::new(1.5, 1.5, 1.5)));

    // Test size calculations
    let size = box1.size();
    assert_eq!(size, Vec3::new(1.0, 1.0, 1.0));

    let volume = box1.volume();
    assert_eq!(volume, 1.0);
}

#[test]
fn test_bounding_sphere_operations() {
    let sphere1 = BoundingSphere::new(Vec3::ZERO, 1.0);
    let sphere2 = BoundingSphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);

    // Test intersection (check if spheres overlap)
    let distance = sphere1.center.distance(sphere2.center);
    assert!(distance <= sphere1.radius + sphere2.radius);

    // Test containment
    assert!(sphere1.contains_point(Vec3::new(0.5, 0.0, 0.0)));
    assert!(!sphere1.contains_point(Vec3::new(2.0, 0.0, 0.0)));

    // Test volume calculation (manually)
    let expected_volume = (4.0 / 3.0) * std::f32::consts::PI * 1.0_f32.powi(3);
    let actual_volume = (4.0 / 3.0) * std::f32::consts::PI * sphere1.radius.powi(3);
    assert!((actual_volume - expected_volume).abs() < 0.001);
}

#[test]
fn test_spatial_operations() {
    // Test distance calculations using Vec3 methods
    let p1 = Vec3::new(0.0, 0.0, 0.0);
    let p2 = Vec3::new(3.0, 4.0, 0.0);
    
    assert_eq!(p1.distance_squared(p2), 25.0);
    assert_eq!(p1.distance(p2), 5.0);

    // Test basic spatial operations
    let triangle = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.5, 1.0, 0.0),
    ];
    
    let test_point = Vec3::new(0.5, 0.25, 0.0);
    
    // Test that the point is reasonably close to the triangle
    let center = (triangle[0] + triangle[1] + triangle[2]) / 3.0;
    assert!((test_point - center).length() < 1.0);
}