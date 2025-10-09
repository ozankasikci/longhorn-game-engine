//! Namespace implementations for the TypeScript API

pub mod engine_world;
pub mod engine_math;
pub mod engine_debug;

// Re-export the registration functions
pub use engine_world::register_engine_world;
pub use engine_math::register_engine_math;
pub use engine_debug::register_engine_debug;