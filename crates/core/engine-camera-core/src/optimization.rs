//! Mobile optimization abstractions for camera systems

use crate::{CameraPerformanceSettings, Result};
use crate::projection::{DeviceInfo, DeviceTier};
use serde::{Serialize, Deserialize};

/// Mobile-specific camera optimizer interface
pub trait MobileOptimizer: Send + Sync {
    /// Optimize camera settings for specific device
    fn optimize_for_device(&mut self, device: &DeviceInfo) -> Result<CameraOptimizations>;
    
    /// Get recommended camera settings
    fn recommended_settings(&self, device: &DeviceInfo) -> CameraPerformanceSettings;
    
    /// Update optimizations based on runtime performance
    fn update_for_performance(&mut self, metrics: &PerformanceMetrics) -> Result<()>;
    
    /// Check if optimizations are enabled
    fn is_enabled(&self) -> bool;
    
    /// Enable or disable optimizations
    fn set_enabled(&mut self, enabled: bool);
    
    /// Get current optimization level
    fn optimization_level(&self) -> OptimizationLevel;
    
    /// Set optimization level
    fn set_optimization_level(&mut self, level: OptimizationLevel);
    
    /// Get device-specific recommendations
    fn device_recommendations(&self, device: &DeviceInfo) -> DeviceRecommendations;
}

/// Device profiler for runtime optimization
pub trait DeviceProfiler: Send + Sync {
    /// Profile device capabilities
    fn profile_device(&mut self) -> Result<DeviceProfile>;
    
    /// Update device profile with runtime data
    fn update_profile(&mut self, metrics: &PerformanceMetrics) -> Result<()>;
    
    /// Get current device profile
    fn current_profile(&self) -> &DeviceProfile;
    
    /// Check if profiling is complete
    fn is_profiling_complete(&self) -> bool;
    
    /// Reset profiling data
    fn reset_profiling(&mut self);
}

/// Performance monitor for camera systems
pub trait PerformanceMonitor: Send + Sync {
    /// Start monitoring performance
    fn start_monitoring(&mut self) -> Result<()>;
    
    /// Stop monitoring and get results
    fn stop_monitoring(&mut self) -> Result<PerformanceMetrics>;
    
    /// Get current performance metrics
    fn current_metrics(&self) -> &PerformanceMetrics;
    
    /// Check if performance is acceptable
    fn is_performance_acceptable(&self, target_fps: f32) -> bool;
    
    /// Get performance trend
    fn performance_trend(&self) -> PerformanceTrend;
}

/// Camera optimizations for mobile devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraOptimizations {
    /// Enable frustum culling optimizations
    pub frustum_culling: bool,
    
    /// Use reduced precision matrices
    pub reduced_precision: bool,
    
    /// Enable LOD bias adjustments
    pub lod_bias_enabled: bool,
    
    /// LOD bias value
    pub lod_bias: f32,
    
    /// Render distance optimization
    pub render_distance: f32,
    
    /// Enable occlusion culling (expensive)
    pub occlusion_culling: bool,
    
    /// Use simplified shaders
    pub simplified_shaders: bool,
    
    /// Reduce shadow quality
    pub reduced_shadows: bool,
    
    /// Enable dynamic resolution scaling
    pub dynamic_resolution: bool,
    
    /// Target resolution scale (0.5 = half resolution)
    pub resolution_scale: f32,
}

/// Performance metrics for optimization decisions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Current frames per second
    pub current_fps: f32,
    
    /// Average FPS over measurement period
    pub average_fps: f32,
    
    /// Minimum FPS recorded
    pub min_fps: f32,
    
    /// Maximum FPS recorded
    pub max_fps: f32,
    
    /// Frame time in milliseconds
    pub frame_time_ms: f32,
    
    /// GPU usage percentage (0-100)
    pub gpu_usage: f32,
    
    /// Memory usage in MB
    pub memory_usage_mb: f32,
    
    /// Render calls per frame
    pub render_calls: u32,
    
    /// Triangles rendered per frame
    pub triangles_rendered: u32,
    
    /// Time spent in culling (microseconds)
    pub culling_time_us: u64,
    
    /// Number of objects culled
    pub objects_culled: u32,
    
    /// Measurement duration in seconds
    pub measurement_duration: f32,
}

/// Device profile for optimization decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    /// Basic device information
    pub device_info: DeviceInfo,
    
    /// Measured performance capabilities
    pub performance_score: f32,
    
    /// Thermal throttling characteristics
    pub thermal_profile: ThermalProfile,
    
    /// Battery usage characteristics
    pub battery_profile: BatteryProfile,
    
    /// Graphics capabilities
    pub graphics_capabilities: GraphicsCapabilities,
    
    /// Memory bandwidth (GB/s)
    pub memory_bandwidth: f32,
    
    /// Fill rate (pixels/second)
    pub fill_rate: f64,
    
    /// Shader ALU performance
    pub shader_performance: f32,
}

/// Thermal behavior profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalProfile {
    /// Temperature at which throttling begins
    pub throttle_temperature: f32,
    
    /// Performance reduction when throttling
    pub throttle_factor: f32,
    
    /// Time to reach throttling under load
    pub time_to_throttle_s: f32,
    
    /// Cooldown time after throttling
    pub cooldown_time_s: f32,
    
    /// Current thermal state
    pub current_state: ThermalState,
}

/// Battery usage characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryProfile {
    /// Power consumption at different performance levels
    pub power_consumption_mw: [f32; 4], // Low, Medium, High, Max
    
    /// Estimated battery life at each level (hours)
    pub battery_life_hours: [f32; 4],
    
    /// Power efficiency score
    pub efficiency_score: f32,
    
    /// Current battery level (0-100)
    pub current_battery_level: f32,
}

/// Graphics hardware capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsCapabilities {
    /// Maximum texture size
    pub max_texture_size: u32,
    
    /// Supported texture formats
    pub texture_formats: Vec<String>,
    
    /// Maximum vertex attributes
    pub max_vertex_attributes: u32,
    
    /// Maximum uniform buffer size
    pub max_uniform_buffer_size: u32,
    
    /// Supports compute shaders
    pub compute_shaders: bool,
    
    /// Supports geometry shaders
    pub geometry_shaders: bool,
    
    /// Supports tessellation
    pub tessellation: bool,
    
    /// Maximum anisotropic filtering
    pub max_anisotropy: f32,
    
    /// Maximum MSAA samples
    pub max_msaa_samples: u32,
}

/// Device-specific recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRecommendations {
    /// Recommended optimization level
    pub optimization_level: OptimizationLevel,
    
    /// Recommended render distance
    pub render_distance: f32,
    
    /// Recommended resolution scale
    pub resolution_scale: f32,
    
    /// Recommended LOD bias
    pub lod_bias: f32,
    
    /// Recommended target FPS
    pub target_fps: f32,
    
    /// Enable specific optimizations
    pub enabled_optimizations: Vec<OptimizationType>,
    
    /// Confidence in recommendations (0-1)
    pub confidence: f32,
    
    /// Reasoning for recommendations
    pub reasoning: String,
}

/// Optimization levels for mobile devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// No optimizations (best quality)
    None,
    
    /// Light optimizations (minor quality trade-offs)
    Light,
    
    /// Balanced optimizations (moderate quality trade-offs)
    Balanced,
    
    /// Aggressive optimizations (significant quality trade-offs)
    Aggressive,
    
    /// Maximum optimizations (minimum quality)
    Maximum,
}

/// Types of optimizations that can be enabled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Frustum culling
    FrustumCulling,
    
    /// Occlusion culling
    OcclusionCulling,
    
    /// Level of detail (LOD)
    LevelOfDetail,
    
    /// Dynamic resolution scaling
    DynamicResolution,
    
    /// Reduced shadow quality
    ReducedShadows,
    
    /// Simplified shaders
    SimplifiedShaders,
    
    /// Reduced texture quality
    ReducedTextures,
    
    /// Lower geometry detail
    ReducedGeometry,
    
    /// Disabled post-processing
    DisabledPostProcessing,
}

/// Thermal states for throttling management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalState {
    /// Normal operating temperature
    Normal,
    
    /// Approaching throttling temperature
    Warm,
    
    /// Throttling in effect
    Throttling,
    
    /// Critical temperature reached
    Critical,
}

/// Performance trends for adaptive optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceTrend {
    /// Performance is improving
    Improving,
    
    /// Performance is stable
    Stable,
    
    /// Performance is declining
    Declining,
    
    /// Performance is unstable/variable
    Unstable,
}

impl Default for CameraOptimizations {
    fn default() -> Self {
        Self {
            frustum_culling: true,
            reduced_precision: false,
            lod_bias_enabled: true,
            lod_bias: 1.0,
            render_distance: 1000.0,
            occlusion_culling: false,
            simplified_shaders: false,
            reduced_shadows: false,
            dynamic_resolution: false,
            resolution_scale: 1.0,
        }
    }
}

impl CameraOptimizations {
    /// Create mobile-optimized settings
    pub fn mobile_optimized() -> Self {
        Self {
            frustum_culling: true,
            reduced_precision: true,
            lod_bias_enabled: true,
            lod_bias: 1.5,
            render_distance: 500.0,
            occlusion_culling: false, // Too expensive for mobile
            simplified_shaders: true,
            reduced_shadows: true,
            dynamic_resolution: true,
            resolution_scale: 0.8,
        }
    }
    
    /// Create aggressive optimization settings
    pub fn aggressive() -> Self {
        Self {
            frustum_culling: true,
            reduced_precision: true,
            lod_bias_enabled: true,
            lod_bias: 2.0,
            render_distance: 300.0,
            occlusion_culling: false,
            simplified_shaders: true,
            reduced_shadows: true,
            dynamic_resolution: true,
            resolution_scale: 0.6,
        }
    }
    
    /// Apply optimizations to camera settings
    pub fn apply_to_settings(&self, settings: &mut CameraPerformanceSettings) {
        settings.frustum_culling_enabled = self.frustum_culling;
        settings.render_distance = self.render_distance;
        
        if self.lod_bias_enabled {
            settings.lod_bias = self.lod_bias;
        }
        
        settings.mobile_optimizations = self.reduced_precision || 
                                       self.simplified_shaders || 
                                       self.reduced_shadows;
    }
}

impl PerformanceMetrics {
    /// Update metrics with new frame data
    pub fn update_frame(&mut self, frame_time_ms: f32) {
        self.frame_time_ms = frame_time_ms;
        self.current_fps = 1000.0 / frame_time_ms.max(0.001);
        
        // Update running averages (simple exponential moving average)
        let alpha = 0.1;
        self.average_fps = self.average_fps * (1.0 - alpha) + self.current_fps * alpha;
        
        self.min_fps = self.min_fps.min(self.current_fps);
        self.max_fps = self.max_fps.max(self.current_fps);
    }
    
    /// Check if performance is below target
    pub fn is_below_target(&self, target_fps: f32) -> bool {
        self.average_fps < target_fps * 0.9 // 10% tolerance
    }
    
    /// Get performance score (0-1, higher is better)
    pub fn performance_score(&self, target_fps: f32) -> f32 {
        (self.average_fps / target_fps).min(1.0)
    }
    
    /// Reset metrics
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

impl Default for DeviceProfile {
    fn default() -> Self {
        Self {
            device_info: DeviceInfo::default(),
            performance_score: 0.5,
            thermal_profile: ThermalProfile::default(),
            battery_profile: BatteryProfile::default(),
            graphics_capabilities: GraphicsCapabilities::default(),
            memory_bandwidth: 25.6, // GB/s - typical mobile
            fill_rate: 1e9, // 1 Gpixel/s
            shader_performance: 0.5,
        }
    }
}

impl Default for ThermalProfile {
    fn default() -> Self {
        Self {
            throttle_temperature: 85.0, // Celsius
            throttle_factor: 0.7, // Reduce to 70% performance
            time_to_throttle_s: 300.0, // 5 minutes
            cooldown_time_s: 120.0, // 2 minutes
            current_state: ThermalState::Normal,
        }
    }
}

impl Default for BatteryProfile {
    fn default() -> Self {
        Self {
            power_consumption_mw: [500.0, 1000.0, 2000.0, 3500.0],
            battery_life_hours: [20.0, 10.0, 5.0, 2.8],
            efficiency_score: 0.5,
            current_battery_level: 100.0,
        }
    }
}

impl Default for GraphicsCapabilities {
    fn default() -> Self {
        Self {
            max_texture_size: 4096,
            texture_formats: vec![
                "RGBA8".to_string(),
                "RGB8".to_string(),
                "RGBA16F".to_string(),
            ],
            max_vertex_attributes: 16,
            max_uniform_buffer_size: 65536,
            compute_shaders: false,
            geometry_shaders: false,
            tessellation: false,
            max_anisotropy: 4.0,
            max_msaa_samples: 4,
        }
    }
}

impl OptimizationLevel {
    /// Get numerical value for comparison (higher = more optimization)
    pub fn value(&self) -> u8 {
        match self {
            OptimizationLevel::None => 0,
            OptimizationLevel::Light => 1,
            OptimizationLevel::Balanced => 2,
            OptimizationLevel::Aggressive => 3,
            OptimizationLevel::Maximum => 4,
        }
    }
    
    /// Get optimization level from device tier
    pub fn from_device_tier(tier: DeviceTier) -> Self {
        match tier {
            DeviceTier::VeryLow => OptimizationLevel::Maximum,
            DeviceTier::Low => OptimizationLevel::Aggressive,
            DeviceTier::Medium => OptimizationLevel::Balanced,
            DeviceTier::High => OptimizationLevel::Light,
            DeviceTier::VeryHigh => OptimizationLevel::None,
        }
    }
}