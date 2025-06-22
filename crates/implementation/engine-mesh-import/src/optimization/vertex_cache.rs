use crate::MeshData;

#[derive(Debug, Clone)]
pub struct CacheOptions {
    pub cache_size: usize,
    pub optimize_overdraw: bool,
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self {
            cache_size: 16,
            optimize_overdraw: false,
        }
    }
}

pub struct VertexCacheOptimizer;

impl VertexCacheOptimizer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn optimize(
        &self,
        mesh: &mut MeshData,
        options: &CacheOptions,
    ) -> Result<MeshData, String> {
        // Simple implementation that reorders indices for better cache locality
        let optimized_indices = self.optimize_indices(&mesh.indices, mesh.vertices.len(), options);
        
        Ok(MeshData {
            name: mesh.name.clone(),
            vertices: mesh.vertices.clone(),
            indices: optimized_indices,
            material: mesh.material.clone(),
        })
    }
    
    pub fn calculate_acmr(&self, indices: &[u32], cache_size: usize) -> f32 {
        if indices.is_empty() {
            return 0.0;
        }
        
        let mut cache = Vec::with_capacity(cache_size);
        let mut cache_misses = 0;
        
        for &index in indices {
            if !cache.contains(&index) {
                cache_misses += 1;
                
                if cache.len() >= cache_size {
                    cache.remove(0);
                }
                cache.push(index);
            } else {
                // Move to end (LRU)
                cache.retain(|&x| x != index);
                cache.push(index);
            }
        }
        
        cache_misses as f32 / (indices.len() as f32 / 3.0)
    }
    
    fn optimize_indices(
        &self,
        indices: &[u32],
        vertex_count: usize,
        _options: &CacheOptions,
    ) -> Vec<u32> {
        // Simple optimization: sort triangles by first vertex index
        // This provides some cache coherency improvement
        let mut triangles: Vec<[u32; 3]> = indices.chunks(3)
            .filter_map(|chunk| {
                if chunk.len() == 3 {
                    Some([chunk[0], chunk[1], chunk[2]])
                } else {
                    None
                }
            })
            .collect();
        
        // Sort by minimum vertex index in each triangle
        triangles.sort_by_key(|tri| tri.iter().min().copied().unwrap_or(0));
        
        // Flatten back to indices
        triangles.into_iter()
            .flat_map(|tri| tri.into_iter())
            .collect()
    }
}