use crate::{MeshData, optimization::OptimizationPipeline, lod::LODGenerator};
use crate::validation::{TopologyValidator, UVValidator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct BatchOptions {
    pub parallel_processing: bool,
    pub optimization_level: OptimizationLevel,
    pub generate_lods: bool,
    pub validate: bool,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            parallel_processing: true,
            optimization_level: OptimizationLevel::Medium,
            generate_lods: false,
            validate: true,
        }
    }
}

#[derive(Debug)]
pub struct BatchResult {
    pub validation_passed: bool,
    pub optimized_mesh: Option<MeshData>,
    pub lod_meshes: Vec<MeshData>,
    pub errors: Vec<String>,
}

pub struct BatchProcessor;

impl BatchProcessor {
    pub fn new() -> Self {
        Self
    }
    
    pub fn process_batch(
        &self,
        meshes: Vec<MeshData>,
        options: &BatchOptions,
    ) -> Result<Vec<BatchResult>, String> {
        let results: Vec<BatchResult> = if options.parallel_processing {
            // In a real implementation, use rayon for parallel processing
            meshes.into_iter()
                .map(|mesh| self.process_single(mesh, options))
                .collect()
        } else {
            meshes.into_iter()
                .map(|mesh| self.process_single(mesh, options))
                .collect()
        };
        
        Ok(results)
    }
    
    fn process_single(&self, mesh: MeshData, options: &BatchOptions) -> BatchResult {
        let mut result = BatchResult {
            validation_passed: true,
            optimized_mesh: None,
            lod_meshes: Vec::new(),
            errors: Vec::new(),
        };
        
        // Validation
        if options.validate {
            let topology_validator = TopologyValidator::new();
            if let Err(e) = topology_validator.validate(&mesh) {
                result.errors.push(format!("Topology validation failed: {}", e));
                result.validation_passed = false;
            }
            
            let uv_validator = UVValidator::new();
            let warnings = uv_validator.get_warnings(&mesh);
            for warning in warnings {
                result.errors.push(format!("UV warning: {}", warning));
            }
        }
        
        // Optimization
        let optimized = match options.optimization_level {
            OptimizationLevel::None => mesh.clone(),
            _ => {
                let mut pipeline = OptimizationPipeline::new();
                let opt_options = crate::optimization::OptimizationOptions {
                    merge_vertices: true,
                    optimize_vertex_cache: options.optimization_level == OptimizationLevel::High,
                    remove_unused_vertices: true,
                    quantize_positions: false,
                    target_index_buffer_size: None,
                };
                
                match pipeline.optimize(mesh.clone(), opt_options) {
                    Ok(optimized) => optimized,
                    Err(e) => {
                        result.errors.push(format!("Optimization failed: {}", e));
                        mesh.clone()
                    }
                }
            }
        };
        
        result.optimized_mesh = Some(optimized.clone());
        
        // LOD generation
        if options.generate_lods {
            let lod_generator = LODGenerator::new();
            let lod_options = crate::lod::LODOptions {
                levels: vec![
                    crate::lod::LODLevel { distance: 10.0, quality: 1.0 },
                    crate::lod::LODLevel { distance: 50.0, quality: 0.5 },
                    crate::lod::LODLevel { distance: 100.0, quality: 0.25 },
                ],
                preserve_boundaries: true,
                preserve_seams: true,
                preserve_uv_boundaries: true,
            };
            
            match lod_generator.generate_lods(&optimized, &lod_options) {
                Ok(lods) => result.lod_meshes = lods,
                Err(e) => result.errors.push(format!("LOD generation failed: {}", e)),
            }
        }
        
        result
    }
}