mod game_loop;
mod application;
mod time_manager;
mod error;
mod system_scheduler;
mod game_context;
mod interpolation;
mod hot_reload;

pub use game_loop::GameLoop;
pub use application::Application;
pub use time_manager::TimeManager;
pub use error::RuntimeError;
pub use system_scheduler::{SystemScheduler, System, SystemError};
pub use game_context::GameContext;
pub use interpolation::{
    InterpolationManager, Interpolatable, InterpolationError,
    Position3D, Rotation3D, Scale3D
};
pub use hot_reload::{
    HotReloadManager, HotReloadEvent, HotReloadError, AssetType
};

pub type Result<T> = std::result::Result<T, RuntimeError>;