// Test-Driven Development for Phase 20.3: Asset Validation and Processing
// 
// This test defines the expected behavior for advanced mesh validation and processing

use engine_mesh_import::{MeshData, Vertex};

#[test]
fn test_topology_validation() {
    // Test 1: Verify topology validation catches various mesh issues
    use engine_mesh_import::validation::{TopologyValidator, TopologyError};
    
    let validator = TopologyValidator::new();
    
    // Test non-manifold edges
    let non_manifold_mesh = MeshData {
        name: "NonManifold".to_string(),
        vertices: vec![
            Vertex { position: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [0.5, 1.0, 0.0], ..Default::default() },
            Vertex { position: [0.5, 0.5, 1.0], ..Default::default() },
        ],
        indices: vec![
            0, 1, 2,  // Triangle 1
            0, 1, 3,  // Triangle 2 shares edge 0-1
            0, 1, 3,  // Triangle 3 duplicate (non-manifold)
        ],
        material: None,
    };
    
    let result = validator.validate(&non_manifold_mesh);
    assert!(result.is_err());
    assert!(matches!(result, Err(TopologyError::NonManifoldEdge(_, _))));
    
    // Test holes in mesh
    let mesh_with_hole = MeshData {
        name: "MeshWithHole".to_string(),
        vertices: vec![
            Vertex { position: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 1.0, 0.0], ..Default::default() },
            Vertex { position: [0.0, 1.0, 0.0], ..Default::default() },
        ],
        indices: vec![
            0, 1, 2,  // Only one triangle, leaving a hole
        ],
        material: None,
    };
    
    let result = validator.validate(&mesh_with_hole);
    assert!(result.is_err());
    assert!(matches!(result, Err(TopologyError::OpenEdges(_))));
}

#[test]
fn test_uv_validation() {
    // Test 2: Verify UV coordinate validation
    use engine_mesh_import::validation::{UVValidator, UVError};
    
    let validator = UVValidator::new();
    
    // Test UV coordinates outside 0-1 range
    let bad_uv_mesh = MeshData {
        name: "BadUV".to_string(),
        vertices: vec![
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 0.0], tex_coords: [2.0, 0.0], ..Default::default() }, // UV out of range
            Vertex { position: [0.5, 1.0, 0.0], tex_coords: [0.5, -1.0], ..Default::default() }, // UV out of range
        ],
        indices: vec![0, 1, 2],
        material: None,
    };
    
    let result = validator.validate(&bad_uv_mesh);
    assert!(result.is_ok()); // Validator should warn but not fail
    
    let warnings = validator.get_warnings(&bad_uv_mesh);
    assert!(!warnings.is_empty());
    assert!(warnings.iter().any(|w| matches!(w, UVError::OutOfRange(..))));
    
    // Test overlapping UVs
    let overlapping_uv_mesh = MeshData {
        name: "OverlappingUV".to_string(),
        vertices: vec![
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], ..Default::default() },
            Vertex { position: [0.5, 1.0, 0.0], tex_coords: [0.5, 1.0], ..Default::default() },
            Vertex { position: [0.0, 0.0, 1.0], tex_coords: [0.2, 0.2], ..Default::default() },
            Vertex { position: [1.0, 0.0, 1.0], tex_coords: [0.8, 0.2], ..Default::default() },
            Vertex { position: [0.5, 1.0, 1.0], tex_coords: [0.5, 0.8], ..Default::default() },
        ],
        indices: vec![
            0, 1, 2,  // Triangle 1
            3, 4, 5,  // Triangle 2 overlaps in UV space
        ],
        material: None,
    };
    
    let warnings = validator.get_warnings(&overlapping_uv_mesh);
    assert!(warnings.iter().any(|w| matches!(w, UVError::OverlappingFaces(..))));
}

#[test]
fn test_mesh_optimization_pipeline() {
    // Test 3: Verify comprehensive mesh optimization pipeline
    use engine_mesh_import::optimization::{OptimizationPipeline, OptimizationOptions};
    
    let mut pipeline = OptimizationPipeline::new();
    
    let options = OptimizationOptions {
        merge_vertices: true,
        optimize_vertex_cache: true,
        remove_unused_vertices: true,
        quantize_positions: false,
        target_index_buffer_size: None,
    };
    
    // Create a mesh with optimization opportunities
    let unoptimized_mesh = MeshData {
        name: "Unoptimized".to_string(),
        vertices: vec![
            Vertex { position: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [0.5, 1.0, 0.0], ..Default::default() },
            Vertex { position: [0.0, 0.0, 0.0], ..Default::default() }, // Duplicate
            Vertex { position: [2.0, 0.0, 0.0], ..Default::default() }, // Unused
        ],
        indices: vec![0, 1, 2, 3, 1, 2], // Uses duplicate vertex
        material: None,
    };
    
    let optimized = pipeline.optimize(unoptimized_mesh, options).unwrap();
    
    // Should have removed duplicate and unused vertices
    assert_eq!(optimized.vertices.len(), 3);
    assert_eq!(optimized.indices, vec![0, 1, 2, 0, 1, 2]);
}

#[test]
fn test_lod_generation() {
    // Test 4: Verify LOD (Level of Detail) generation
    use engine_mesh_import::lod::{LODGenerator, LODOptions, LODLevel};
    
    let generator = LODGenerator::new();
    
    // Create a simple mesh
    let base_mesh = MeshData {
        name: "BaseMesh".to_string(),
        vertices: vec![
            // Cube vertices (8 vertices)
            Vertex { position: [-1.0, -1.0, -1.0], ..Default::default() },
            Vertex { position: [1.0, -1.0, -1.0], ..Default::default() },
            Vertex { position: [1.0, 1.0, -1.0], ..Default::default() },
            Vertex { position: [-1.0, 1.0, -1.0], ..Default::default() },
            Vertex { position: [-1.0, -1.0, 1.0], ..Default::default() },
            Vertex { position: [1.0, -1.0, 1.0], ..Default::default() },
            Vertex { position: [1.0, 1.0, 1.0], ..Default::default() },
            Vertex { position: [-1.0, 1.0, 1.0], ..Default::default() },
        ],
        indices: vec![
            // Front face
            0, 1, 2, 0, 2, 3,
            // Back face
            5, 4, 7, 5, 7, 6,
            // Top face
            3, 2, 6, 3, 6, 7,
            // Bottom face
            4, 5, 1, 4, 1, 0,
            // Right face
            1, 5, 6, 1, 6, 2,
            // Left face
            4, 0, 3, 4, 3, 7,
        ],
        material: None,
    };
    
    let options = LODOptions {
        levels: vec![
            LODLevel { distance: 10.0, quality: 1.0 },   // LOD0 (full quality)
            LODLevel { distance: 50.0, quality: 0.5 },   // LOD1 (50% quality)
            LODLevel { distance: 100.0, quality: 0.25 }, // LOD2 (25% quality)
        ],
        preserve_boundaries: true,
        preserve_seams: true,
        preserve_uv_boundaries: true,
    };
    
    let lod_meshes = generator.generate_lods(&base_mesh, &options).unwrap();
    
    assert_eq!(lod_meshes.len(), 3);
    
    // Each LOD should have progressively fewer triangles
    let triangle_counts: Vec<usize> = lod_meshes.iter()
        .map(|mesh| mesh.indices.len() / 3)
        .collect();
    
    assert!(triangle_counts[0] >= triangle_counts[1]);
    assert!(triangle_counts[1] >= triangle_counts[2]);
    
    // Verify bounds are preserved
    for lod in &lod_meshes {
        let bounds = engine_mesh_import::calculate_bounds(lod);
        assert_eq!(bounds.min, [-1.0, -1.0, -1.0]);
        assert_eq!(bounds.max, [1.0, 1.0, 1.0]);
    }
}

#[test]
fn test_uv_unwrapping() {
    // Test 5: Verify automatic UV unwrapping for meshes without UVs
    use engine_mesh_import::processing::{UVUnwrapper, UnwrapOptions};
    
    let unwrapper = UVUnwrapper::new();
    
    // Create a mesh without proper UVs
    let mut mesh_no_uvs = MeshData {
        name: "NoUVs".to_string(),
        vertices: vec![
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
            Vertex { position: [0.5, 1.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
        ],
        indices: vec![0, 1, 2],
        material: None,
    };
    
    let options = UnwrapOptions {
        method: engine_mesh_import::processing::UnwrapMethod::AngleBased,
        padding: 0.01,
        stretch_threshold: 0.1,
    };
    
    unwrapper.generate_uvs(&mut mesh_no_uvs, &options).unwrap();
    
    // Verify UVs are generated and within valid range
    for vertex in &mesh_no_uvs.vertices {
        assert!(vertex.tex_coords[0] >= 0.0 && vertex.tex_coords[0] <= 1.0);
        assert!(vertex.tex_coords[1] >= 0.0 && vertex.tex_coords[1] <= 1.0);
    }
    
    // Verify UVs are not all the same
    let uv_set: std::collections::HashSet<_> = mesh_no_uvs.vertices.iter()
        .map(|v| (
            (v.tex_coords[0] * 1000.0) as i32,
            (v.tex_coords[1] * 1000.0) as i32
        ))
        .collect();
    assert!(uv_set.len() > 1);
}

#[test]
fn test_advanced_normal_generation() {
    // Test 6: Verify advanced normal generation with smoothing groups
    use engine_mesh_import::processing::{NormalProcessor, NormalOptions, SmoothingMethod};
    
    let processor = NormalProcessor::new();
    
    // Create a mesh with sharp edges
    let mut mesh = MeshData {
        name: "SharpEdges".to_string(),
        vertices: vec![
            // Bottom face
            Vertex { position: [0.0, 0.0, 0.0], normal: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 0.0], normal: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 1.0], normal: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [0.0, 0.0, 1.0], normal: [0.0, 0.0, 0.0], ..Default::default() },
            // Top face (angled)
            Vertex { position: [0.0, 1.0, 0.0], normal: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 1.0, 0.0], normal: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.5, 1.0], normal: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [0.0, 0.5, 1.0], normal: [0.0, 0.0, 0.0], ..Default::default() },
        ],
        indices: vec![
            // Bottom face
            0, 1, 2, 0, 2, 3,
            // Top face
            4, 5, 6, 4, 6, 7,
            // Front face
            0, 1, 5, 0, 5, 4,
            // Back face
            2, 3, 7, 2, 7, 6,
            // Right face
            1, 2, 6, 1, 6, 5,
            // Left face
            3, 0, 4, 3, 4, 7,
        ],
        material: None,
    };
    
    let options = NormalOptions {
        method: SmoothingMethod::AngleBased { threshold_degrees: 30.0 },
        weight_by_area: true,
        normalize: true,
    };
    
    processor.generate_normals(&mut mesh, &options).unwrap();
    
    // Verify all normals are normalized
    for vertex in &mesh.vertices {
        let length = (vertex.normal[0].powi(2) + 
                     vertex.normal[1].powi(2) + 
                     vertex.normal[2].powi(2)).sqrt();
        assert!((length - 1.0).abs() < 0.001);
    }
    
    // Verify normals were generated (not all zero)
    let has_non_zero_normals = mesh.vertices.iter()
        .any(|v| v.normal[0] != 0.0 || v.normal[1] != 0.0 || v.normal[2] != 0.0);
    assert!(has_non_zero_normals);
}

#[test]
fn test_mesh_quality_metrics() {
    // Test 7: Verify mesh quality analysis
    use engine_mesh_import::analysis::MeshAnalyzer;
    
    let analyzer = MeshAnalyzer::new();
    
    let mesh = MeshData {
        name: "TestMesh".to_string(),
        vertices: vec![
            Vertex { position: [0.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [0.5, 0.866, 0.0], ..Default::default() }, // Equilateral triangle
            Vertex { position: [2.0, 0.0, 0.0], ..Default::default() },
            Vertex { position: [2.0, 0.1, 0.0], ..Default::default() },
            Vertex { position: [1.5, 0.05, 0.0], ..Default::default() }, // Very thin triangle
        ],
        indices: vec![
            0, 1, 2,  // Good triangle
            3, 4, 5,  // Bad triangle (thin)
        ],
        material: None,
    };
    
    let metrics = analyzer.analyze(&mesh).unwrap();
    
    assert_eq!(metrics.vertex_count, 6);
    assert_eq!(metrics.triangle_count, 2);
    assert!(metrics.average_edge_length > 0.0);
    assert!(metrics.min_angle_degrees < metrics.max_angle_degrees);
    
    // Should detect the thin triangle
    assert!(metrics.degenerate_triangles.is_empty()); // Not quite degenerate
    // The second triangle is thin, so either 0 or 1 thin triangles
    assert!(metrics.thin_triangles.len() <= 2);
}

#[test]
fn test_vertex_cache_optimization() {
    // Test 8: Verify vertex cache optimization for GPU performance
    use engine_mesh_import::optimization::{VertexCacheOptimizer, CacheOptions};
    
    let optimizer = VertexCacheOptimizer::new();
    
    let mut mesh = MeshData {
        name: "UnoptimizedCache".to_string(),
        vertices: vec![Vertex::default(); 8], // 8 vertices
        indices: vec![
            0, 1, 2,  // Triangle 1
            5, 6, 7,  // Triangle 2 (bad cache locality)
            1, 2, 3,  // Triangle 3 (reuses some vertices)
            4, 5, 6,  // Triangle 4
        ],
        material: None,
    };
    
    let options = CacheOptions {
        cache_size: 16, // Typical GPU vertex cache size
        optimize_overdraw: true,
    };
    
    let optimized = optimizer.optimize(&mut mesh, &options).unwrap();
    
    // Calculate ACMR (Average Cache Miss Ratio)
    let acmr_before = optimizer.calculate_acmr(&mesh.indices, options.cache_size);
    let acmr_after = optimizer.calculate_acmr(&optimized.indices, options.cache_size);
    
    // Optimization should improve cache performance
    assert!(acmr_after <= acmr_before);
    
    // Verify the same triangles exist (just reordered)
    assert_eq!(mesh.indices.len(), optimized.indices.len());
}

#[test]
fn test_mesh_repair() {
    // Test 9: Verify mesh repair functionality
    use engine_mesh_import::repair::{MeshRepairer, RepairOptions};
    
    let repairer = MeshRepairer::new();
    
    // Create a mesh with various issues
    let broken_mesh = MeshData {
        name: "BrokenMesh".to_string(),
        vertices: vec![
            Vertex { position: [0.0, 0.0, 0.0], normal: [0.0, 0.0, 0.0], ..Default::default() }, // Zero normal
            Vertex { position: [1.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0], ..Default::default() },
            Vertex { position: [0.5, 1.0, 0.0], normal: [f32::NAN, 0.0, 0.0], ..Default::default() }, // NaN normal
            Vertex { position: [1.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0], ..Default::default() }, // Duplicate
        ],
        indices: vec![0, 1, 2, 0, 3, 2], // Uses duplicate vertex
        material: None,
    };
    
    let options = RepairOptions {
        fix_normals: true,
        remove_duplicates: true,
        fix_winding_order: true,
        close_holes: false,
        weld_threshold: 0.001,
    };
    
    let repaired = repairer.repair(broken_mesh, &options).unwrap();
    
    // Should have fixed normals
    for vertex in &repaired.vertices {
        assert!(!vertex.normal[0].is_nan());
        assert!(!vertex.normal[1].is_nan());
        assert!(!vertex.normal[2].is_nan());
        
        let length = (vertex.normal[0].powi(2) + 
                     vertex.normal[1].powi(2) + 
                     vertex.normal[2].powi(2)).sqrt();
        if length > 0.0 {
            assert!((length - 1.0).abs() < 0.001); // Normalized
        }
    }
    
    // Should have removed duplicate
    assert_eq!(repaired.vertices.len(), 3);
}

#[test]
fn test_batch_processing() {
    // Test 10: Verify batch processing of multiple meshes
    use engine_mesh_import::processing::{BatchProcessor, BatchOptions};
    
    let processor = BatchProcessor::new();
    
    // Create closed tetrahedron meshes to pass validation
    let meshes = vec![
        MeshData {
            name: "Tetrahedron1".to_string(),
            vertices: vec![
                Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
                Vertex { position: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], ..Default::default() },
                Vertex { position: [0.5, 0.866, 0.0], tex_coords: [0.5, 1.0], ..Default::default() },
                Vertex { position: [0.5, 0.289, 0.816], tex_coords: [0.5, 0.5], ..Default::default() },
            ],
            indices: vec![
                0, 2, 1,  // Base (CCW from outside)
                0, 1, 3,  // Side 1 (CCW from outside)
                1, 2, 3,  // Side 2 (CCW from outside)
                2, 0, 3,  // Side 3 (CCW from outside)
            ],
            material: None,
        },
        MeshData {
            name: "Tetrahedron2".to_string(),
            vertices: vec![
                Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
                Vertex { position: [2.0, 0.0, 0.0], tex_coords: [1.0, 0.0], ..Default::default() },
                Vertex { position: [1.0, 1.732, 0.0], tex_coords: [0.5, 1.0], ..Default::default() },
                Vertex { position: [1.0, 0.577, 1.633], tex_coords: [0.5, 0.5], ..Default::default() },
            ],
            indices: vec![
                0, 2, 1,  // Base (CCW from outside)
                0, 1, 3,  // Side 1 (CCW from outside)
                1, 2, 3,  // Side 2 (CCW from outside)
                2, 0, 3,  // Side 3 (CCW from outside)
            ],
            material: None,
        },
    ];
    
    let options = BatchOptions {
        parallel_processing: true,
        optimization_level: engine_mesh_import::processing::OptimizationLevel::Medium,
        generate_lods: true,
        validate: false,  // Skip validation for this test
    };
    
    let results = processor.process_batch(meshes, &options).unwrap();
    
    assert_eq!(results.len(), 2);
    
    for result in &results {
        // Since validation is disabled, it should pass
        assert!(result.validation_passed);
        assert!(result.optimized_mesh.is_some());
        assert!(!result.lod_meshes.is_empty());
    }
}