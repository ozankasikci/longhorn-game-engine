//! System information and platform detection

/// Platform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Ios,
    Android,
    Web,
}

/// System information
pub struct SystemInfo {
    pub platform: Platform,
    pub os_version: String,
    pub cpu_count: usize,
    pub memory_total: u64,
}

impl SystemInfo {
    /// Get system information
    pub fn get() -> SystemInfo {
        Self {
            platform: Self::detect_platform(),
            os_version: "Unknown".to_string(),
            cpu_count: num_cpus::get(),
            memory_total: 0, // TODO: Implement memory detection
        }
    }

    /// Detect the current platform
    pub fn detect_platform() -> Platform {
        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "ios")]
        return Platform::Ios;

        #[cfg(target_os = "android")]
        return Platform::Android;

        #[cfg(target_arch = "wasm32")]
        return Platform::Web;

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "ios",
            target_os = "android",
            target_arch = "wasm32"
        )))]
        return Platform::Linux; // Default fallback
    }
}
