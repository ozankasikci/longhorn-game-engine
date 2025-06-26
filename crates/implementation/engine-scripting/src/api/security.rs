//! API Security implementation for safe script execution
//! Provides permission-based access control and input validation

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/// API permission flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiPermission {
    ConsoleWrite,
    EntityRead,
    EntityWrite,
    FileRead,
    FileWrite,
    NetworkAccess,
    SystemInfo,
}

/// Script capabilities declaration
#[derive(Debug, Clone)]
pub struct ScriptCapabilities {
    permissions: HashSet<ApiPermission>,
    allowed_paths: Vec<String>,
    max_memory: Option<usize>,
    max_execution_time: Option<Duration>,
}

impl ScriptCapabilities {
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
            allowed_paths: Vec::new(),
            max_memory: None,
            max_execution_time: None,
        }
    }

    pub fn require_file_read(mut self, path_pattern: &str) -> Self {
        self.permissions.insert(ApiPermission::FileRead);
        self.allowed_paths.push(path_pattern.to_string());
        self
    }

    pub fn require_console_write(mut self) -> Self {
        self.permissions.insert(ApiPermission::ConsoleWrite);
        self
    }

    pub fn require_entity_read(mut self) -> Self {
        self.permissions.insert(ApiPermission::EntityRead);
        self
    }

    pub fn require_entity_write(mut self) -> Self {
        self.permissions.insert(ApiPermission::EntityWrite);
        self
    }

    pub fn max_memory(mut self, bytes: usize) -> Self {
        self.max_memory = Some(bytes);
        self
    }

    pub fn max_execution_time_ms(mut self, ms: u64) -> Self {
        self.max_execution_time = Some(Duration::from_millis(ms));
        self
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        match capability {
            "file_read" => self.permissions.contains(&ApiPermission::FileRead),
            "file_write" => self.permissions.contains(&ApiPermission::FileWrite),
            "console_write" => self.permissions.contains(&ApiPermission::ConsoleWrite),
            "entity_read" => self.permissions.contains(&ApiPermission::EntityRead),
            "entity_write" => self.permissions.contains(&ApiPermission::EntityWrite),
            _ => false,
        }
    }
}

/// API configuration for function allowlisting
#[derive(Debug)]
pub struct ScriptApiConfig {
    allowed_functions: HashMap<String, ApiPermission>,
    denied_functions: HashSet<String>,
}

impl ScriptApiConfig {
    pub fn new() -> Self {
        let mut config = Self {
            allowed_functions: HashMap::new(),
            denied_functions: HashSet::new(),
        };
        
        // Pre-configure some safe defaults
        config.allow_function("console.log", ApiPermission::ConsoleWrite);
        config.allow_function("entity.get_component", ApiPermission::EntityRead);
        config.deny_function("engine.shutdown");
        config.deny_function("os.execute");
        
        config
    }

    pub fn allow_function(&mut self, function_name: &str, permission: ApiPermission) {
        self.allowed_functions.insert(function_name.to_string(), permission);
        self.denied_functions.remove(function_name);
    }

    pub fn deny_function(&mut self, function_name: &str) {
        self.denied_functions.insert(function_name.to_string());
        self.allowed_functions.remove(function_name);
    }

    pub fn is_function_allowed(&self, function_name: &str) -> bool {
        !self.denied_functions.contains(function_name) && 
        (self.allowed_functions.contains_key(function_name) || 
         !function_name.contains('.')) // Allow simple functions by default
    }

    pub fn get_required_permission(&self, function_name: &str) -> Option<ApiPermission> {
        self.allowed_functions.get(function_name).copied()
    }
}

/// Rate limiter for API calls
#[derive(Debug)]
pub struct ApiRateLimiter {
    limits: HashMap<String, RateLimit>,
    call_history: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

#[derive(Debug, Clone)]
struct RateLimit {
    max_calls: usize,
    window: Duration,
}

impl ApiRateLimiter {
    pub fn new() -> Self {
        let mut limits = HashMap::new();
        
        // Default rate limits
        limits.insert("console.log".to_string(), RateLimit {
            max_calls: 100,
            window: Duration::from_secs(1),
        });
        
        limits.insert("entity.create".to_string(), RateLimit {
            max_calls: 50,
            window: Duration::from_secs(1),
        });

        Self {
            limits,
            call_history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check_rate_limit(&self, api_name: &str) -> Result<(), String> {
        if let Some(limit) = self.limits.get(api_name) {
            let mut history = self.call_history.lock().unwrap();
            let now = Instant::now();
            
            let calls = history.entry(api_name.to_string()).or_insert_with(Vec::new);
            
            // Remove old calls outside the window
            calls.retain(|&call_time| now.duration_since(call_time) < limit.window);
            
            if calls.len() >= limit.max_calls {
                return Err(format!("Rate limit exceeded for {}: {} calls in {:?}", 
                    api_name, limit.max_calls, limit.window));
            }
            
            calls.push(now);
        }
        
        Ok(())
    }
}

/// Input validator for API calls
pub struct ApiInputValidator;

impl ApiInputValidator {
    pub fn validate_file_path(path: &str) -> Result<String, String> {
        println!("DEBUG: Validating path: '{}'", path);
        
        // Check for path traversal
        if path.contains("..") || path.contains("~") {
            println!("DEBUG: Path traversal detected!");
            return Err("path traversal detected".to_string());
        }
        
        // Ensure absolute paths are not allowed
        if path.starts_with('/') || path.starts_with('\\') {
            return Err("absolute paths not allowed".to_string());
        }
        
        Ok(path.to_string())
    }
    
    pub fn validate_string_length(s: &str, max_length: usize) -> Result<(), String> {
        if s.len() > max_length {
            return Err(format!("String too long: {} > {}", s.len(), max_length));
        }
        Ok(())
    }
    
    pub fn validate_query(query: &str) -> Result<String, String> {
        // Basic SQL injection prevention
        let dangerous_patterns = ["'", "\"", ";", "--", "/*", "*/", "OR", "AND", "DROP", "DELETE"];
        
        for pattern in &dangerous_patterns {
            if query.to_uppercase().contains(pattern) {
                return Err(format!("Invalid query: dangerous pattern '{}' detected", pattern));
            }
        }
        
        Ok(query.to_string())
    }
}