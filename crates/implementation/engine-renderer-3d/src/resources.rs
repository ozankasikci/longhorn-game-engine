//! Resource management for the 3D renderer
//! 
//! This module handles efficient GPU resource allocation, buffer management,
//! and resource pooling for optimal performance.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use wgpu::{Device, Queue, Buffer, BufferUsages, util::DeviceExt};
use crate::{Vertex, Mesh, Material};
use crate::texture::{TextureManager, TextureDescriptor};

/// Resource manager for handling GPU buffers and materials
pub struct ResourceManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    
    // Mesh resources
    meshes: RwLock<HashMap<u32, MeshResource>>,
    next_mesh_id: std::sync::atomic::AtomicU32,
    
    // Material resources
    materials: RwLock<HashMap<u32, MaterialResource>>,
    next_material_id: std::sync::atomic::AtomicU32,
    
    // Texture management
    texture_manager: TextureManager,
    
    // Buffer pools for reuse
    vertex_buffer_pool: RwLock<Vec<Buffer>>,
    index_buffer_pool: RwLock<Vec<Buffer>>,
    uniform_buffer_pool: RwLock<Vec<Buffer>>,
}

/// GPU resources for a mesh
#[derive(Debug)]
pub struct MeshResource {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
    pub name: String,
}

/// GPU resources for a material
#[derive(Debug)]
pub struct MaterialResource {
    pub uniform_buffer: Buffer,
    pub bind_group: wgpu::BindGroup,
    pub material_data: Material,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let texture_manager = TextureManager::new(device.clone(), queue.clone());
        
        Self {
            device,
            queue,
            meshes: RwLock::new(HashMap::new()),
            next_mesh_id: std::sync::atomic::AtomicU32::new(0),
            materials: RwLock::new(HashMap::new()),
            next_material_id: std::sync::atomic::AtomicU32::new(0),
            texture_manager,
            vertex_buffer_pool: RwLock::new(Vec::new()),
            index_buffer_pool: RwLock::new(Vec::new()),
            uniform_buffer_pool: RwLock::new(Vec::new()),
        }
    }
    
    /// Upload a mesh to GPU and return its ID
    pub fn upload_mesh(&self, mesh: Mesh) -> Result<u32, anyhow::Error> {
        let mesh_id = self.next_mesh_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Create vertex buffer
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Vertex Buffer {}", mesh.name)),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: BufferUsages::VERTEX,
        });
        
        // Create index buffer
        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Index Buffer {}", mesh.name)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: BufferUsages::INDEX,
        });
        
        let mesh_resource = MeshResource {
            vertex_buffer,
            index_buffer,
            vertex_count: mesh.vertices.len() as u32,
            index_count: mesh.indices.len() as u32,
            name: mesh.name,
        };
        
        // Store the mesh resource
        self.meshes.write().unwrap().insert(mesh_id, mesh_resource);
        
        log::info!("Uploaded mesh {} with ID {}", mesh_id, mesh_id);
        Ok(mesh_id)
    }
    
    /// Upload a material to GPU and return its ID
    pub fn upload_material(&self, material: Material, bind_group_layout: &wgpu::BindGroupLayout) -> Result<u32, anyhow::Error> {
        let material_id = self.next_material_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Create material uniform data
        #[repr(C)]
        #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct MaterialUniform {
            albedo: [f32; 3],
            metallic: f32,
            roughness: f32,
            _padding: [f32; 3], // Align to 16 bytes
        }
        
        let uniform_data = MaterialUniform {
            albedo: material.albedo.to_array(),
            metallic: material.metallic,
            roughness: material.roughness,
            _padding: [0.0; 3],
        };
        
        // Create uniform buffer
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Material Buffer {}", material.name)),
            contents: bytemuck::cast_slice(&[uniform_data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Material Bind Group {}", material.name)),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
        });
        
        let material_resource = MaterialResource {
            uniform_buffer,
            bind_group,
            material_data: material,
        };
        
        // Store the material resource
        self.materials.write().unwrap().insert(material_id, material_resource);
        
        log::info!("Uploaded material {} with ID {}", material_id, material_id);
        Ok(material_id)
    }
    
    /// Get mesh resource by ID
    pub fn get_mesh(&self, mesh_id: u32) -> Option<std::sync::RwLockReadGuard<HashMap<u32, MeshResource>>> {
        let meshes = self.meshes.read().unwrap();
        if meshes.contains_key(&mesh_id) {
            Some(meshes)
        } else {
            None
        }
    }
    
    /// Get material resource by ID
    pub fn get_material(&self, material_id: u32) -> Option<std::sync::RwLockReadGuard<HashMap<u32, MaterialResource>>> {
        let materials = self.materials.read().unwrap();
        if materials.contains_key(&material_id) {
            Some(materials)
        } else {
            None
        }
    }
    
    /// Update material properties
    pub fn update_material(&self, material_id: u32, material: Material) -> Result<(), anyhow::Error> {
        let materials = self.materials.read().unwrap();
        if let Some(material_resource) = materials.get(&material_id) {
            // Update uniform data
            #[repr(C)]
            #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
            struct MaterialUniform {
                albedo: [f32; 3],
                metallic: f32,
                roughness: f32,
                _padding: [f32; 3],
            }
            
            let uniform_data = MaterialUniform {
                albedo: material.albedo.to_array(),
                metallic: material.metallic,
                roughness: material.roughness,
                _padding: [0.0; 3],
            };
            
            self.queue.write_buffer(
                &material_resource.uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniform_data]),
            );
            
            log::debug!("Updated material {}", material_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Material {} not found", material_id))
        }
    }
    
    /// Load default meshes (triangle, cube, etc.)
    pub fn load_default_meshes(&self) -> Result<HashMap<String, u32>, anyhow::Error> {
        let mut default_meshes = HashMap::new();
        
        // Load triangle
        let triangle = Mesh::triangle();
        let triangle_id = self.upload_mesh(triangle)?;
        default_meshes.insert("triangle".to_string(), triangle_id);
        
        // Load cube
        let cube = Mesh::cube();
        let cube_id = self.upload_mesh(cube)?;
        default_meshes.insert("cube".to_string(), cube_id);
        
        log::info!("Loaded {} default meshes", default_meshes.len());
        Ok(default_meshes)
    }
    
    /// Load default materials
    pub fn load_default_materials(&self, bind_group_layout: &wgpu::BindGroupLayout) -> Result<HashMap<String, u32>, anyhow::Error> {
        let mut default_materials = HashMap::new();
        
        // Default material
        let default_material = Material::new("Default".to_string());
        let default_id = self.upload_material(default_material, bind_group_layout)?;
        default_materials.insert("default".to_string(), default_id);
        
        // Red material
        let red_material = Material::red();
        let red_id = self.upload_material(red_material, bind_group_layout)?;
        default_materials.insert("red".to_string(), red_id);
        
        // Green material
        let green_material = Material::green();
        let green_id = self.upload_material(green_material, bind_group_layout)?;
        default_materials.insert("green".to_string(), green_id);
        
        // Blue material
        let blue_material = Material::blue();
        let blue_id = self.upload_material(blue_material, bind_group_layout)?;
        default_materials.insert("blue".to_string(), blue_id);
        
        log::info!("Loaded {} default materials", default_materials.len());
        Ok(default_materials)
    }
    
    /// Load default textures
    pub fn load_default_textures(&self) -> Result<HashMap<String, u32>, anyhow::Error> {
        self.texture_manager.load_default_textures()
    }
    
    /// Create a texture from descriptor
    pub fn create_texture(&self, desc: TextureDescriptor) -> Result<u32, anyhow::Error> {
        self.texture_manager.create_texture(desc)
    }
    
    /// Create a solid color texture
    pub fn create_solid_color_texture(&self, color: [u8; 4], size: u32) -> Result<u32, anyhow::Error> {
        self.texture_manager.create_solid_color_texture(color, size)
    }
    
    /// Get texture by ID
    pub fn get_texture(&self, texture_id: u32) -> Option<std::sync::RwLockReadGuard<std::collections::HashMap<u32, crate::texture::TextureResource>>> {
        self.texture_manager.get_texture(texture_id)
    }
    
    /// Get resource statistics
    pub fn get_stats(&self) -> ResourceStats {
        let meshes = self.meshes.read().unwrap();
        let materials = self.materials.read().unwrap();
        
        ResourceStats {
            mesh_count: meshes.len(),
            material_count: materials.len(),
            texture_count: self.texture_manager.texture_count(),
            vertex_buffer_pool_size: self.vertex_buffer_pool.read().unwrap().len(),
            index_buffer_pool_size: self.index_buffer_pool.read().unwrap().len(),
            uniform_buffer_pool_size: self.uniform_buffer_pool.read().unwrap().len(),
        }
    }
    
    /// Clean up unused resources (placeholder for future optimization)
    pub fn cleanup_unused(&self) {
        // TODO: Implement reference counting and cleanup
        log::debug!("Resource cleanup requested (not yet implemented)");
    }
}

/// Statistics about resource usage
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub mesh_count: usize,
    pub material_count: usize,
    pub texture_count: usize,
    pub vertex_buffer_pool_size: usize,
    pub index_buffer_pool_size: usize,
    pub uniform_buffer_pool_size: usize,
}

impl std::fmt::Display for ResourceStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "ResourceStats {{ meshes: {}, materials: {}, textures: {}, pools: [{}, {}, {}] }}",
            self.mesh_count,
            self.material_count,
            self.texture_count,
            self.vertex_buffer_pool_size,
            self.index_buffer_pool_size,
            self.uniform_buffer_pool_size
        )
    }
}