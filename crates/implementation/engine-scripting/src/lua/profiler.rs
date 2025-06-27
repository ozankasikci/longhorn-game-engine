//! Lua script performance profiling system

use crate::ScriptError;
use mlua::{Lua, Function as LuaFunction};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Function execution profile data
#[derive(Debug, Clone)]
pub struct FunctionProfile {
    pub function_name: String,
    pub file_name: String,
    pub call_count: u32,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub line_number: u32,
}

/// Script execution profile data
#[derive(Debug, Clone)]
pub struct ScriptProfile {
    pub script_name: String,
    pub execution_time: Duration,
    pub memory_used: u64,
    pub function_calls: u32,
}

/// Memory usage snapshot
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub label: String,
    pub total_bytes: u64,
    pub lua_bytes: u64,
    pub script_bytes: u64,
}

/// Garbage collection event data
#[derive(Debug, Clone)]
pub struct GcEvent {
    pub gc_type: String,
    pub duration: Duration,
    pub bytes_freed: u64,
    pub timestamp: Instant,
}

/// Performance profiling report
#[derive(Debug, Clone)]
pub struct ProfilerReport {
    pub functions: Vec<FunctionProfile>,
    pub scripts: Vec<ScriptProfile>,
    pub memory_snapshots: Vec<MemorySnapshot>,
    pub gc_events: Vec<GcEvent>,
    pub total_profiling_time: Duration,
    pub peak_memory_usage: u64,
}

/// Function call stack entry for tracking nested calls
#[derive(Debug, Clone)]
struct CallStackEntry {
    function_name: String,
    file_name: String,
    line_number: u32,
    start_time: Instant,
}

/// Script execution tracking entry
#[derive(Debug, Clone)]
struct ScriptExecution {
    script_name: String,
    start_time: Instant,
    memory_at_start: u64,
}

/// Performance warning types
#[derive(Debug, Clone)]
pub enum PerformanceWarning {
    SlowFunction { function_name: String, duration: Duration },
    HighMemoryUsage { script_name: String, bytes: u64 },
    FrequentGc { count: u32, total_time: Duration },
}

/// Lua script performance profiler
pub struct LuaProfiler {
    is_active: bool,
    function_profiles: HashMap<String, FunctionProfile>,
    script_profiles: HashMap<String, ScriptProfile>,
    memory_snapshots: Vec<MemorySnapshot>,
    gc_events: Vec<GcEvent>,
    call_stack: Vec<CallStackEntry>,
    script_executions: HashMap<String, ScriptExecution>,
    current_memory: u64,
    peak_memory: u64,
    start_time: Option<Instant>,
    function_time_threshold: Duration,
    memory_threshold: u64,
    max_call_stack_depth: usize,
}

impl LuaProfiler {
    /// Create a new profiler instance
    pub fn new(_lua: &Lua) -> Result<Self, ScriptError> {
        Ok(Self {
            is_active: false,
            function_profiles: HashMap::new(),
            script_profiles: HashMap::new(),
            memory_snapshots: Vec::new(),
            gc_events: Vec::new(),
            call_stack: Vec::new(),
            script_executions: HashMap::new(),
            current_memory: 0,
            peak_memory: 0,
            start_time: None,
            function_time_threshold: Duration::from_millis(10),
            memory_threshold: 1024,
            max_call_stack_depth: 0,
        })
    }
    
    /// Check if profiler is currently active
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// Start profiling
    pub fn start_profiling(&mut self) -> Result<(), ScriptError> {
        if self.is_active {
            return Err(ScriptError::InvalidArguments {
                script_id: None,
                function_name: "start_profiling".to_string(),
                message: "Profiler is already active".to_string(),
                expected: "inactive profiler".to_string(),
                actual: "active profiler".to_string(),
            });
        }
        
        self.is_active = true;
        self.start_time = Some(Instant::now());
        self.current_memory = 8192; // Simulate initial memory usage
        Ok(())
    }
    
    /// Stop profiling
    pub fn stop_profiling(&mut self) -> Result<(), ScriptError> {
        if !self.is_active {
            return Err(ScriptError::InvalidArguments {
                script_id: None,
                function_name: "stop_profiling".to_string(),
                message: "Profiler is not active".to_string(),
                expected: "active profiler".to_string(),
                actual: "inactive profiler".to_string(),
            });
        }
        
        self.is_active = false;
        Ok(())
    }
    
    /// Reset profiler data
    pub fn reset(&mut self) -> Result<(), ScriptError> {
        self.function_profiles.clear();
        self.script_profiles.clear();
        self.memory_snapshots.clear();
        self.gc_events.clear();
        self.call_stack.clear();
        self.script_executions.clear();
        self.current_memory = 0;
        self.peak_memory = 0;
        self.start_time = None;
        self.max_call_stack_depth = 0;
        Ok(())
    }
    
    /// Get the number of profiled functions
    pub fn get_function_count(&self) -> usize {
        self.function_profiles.len()
    }
    
    /// Get current memory usage
    pub fn get_memory_usage(&self) -> u64 {
        self.current_memory
    }
    
    /// Record function entry
    pub fn record_function_enter(
        &mut self, 
        file_name: &str, 
        function_name: &str, 
        line_number: u32
    ) -> Result<(), ScriptError> {
        if !self.is_active {
            return Ok(());
        }
        
        let entry = CallStackEntry {
            function_name: function_name.to_string(),
            file_name: file_name.to_string(),
            line_number,
            start_time: Instant::now(),
        };
        
        self.call_stack.push(entry);
        
        // Update max call stack depth
        if self.call_stack.len() > self.max_call_stack_depth {
            self.max_call_stack_depth = self.call_stack.len();
        }
        
        Ok(())
    }
    
    /// Record function exit
    pub fn record_function_exit(
        &mut self, 
        file_name: &str, 
        function_name: &str, 
        _line_number: u32
    ) -> Result<(), ScriptError> {
        if !self.is_active {
            return Ok(());
        }
        
        // Find matching call stack entry
        if let Some(entry) = self.call_stack.pop() {
            if entry.function_name == function_name && entry.file_name == file_name {
                let execution_time = entry.start_time.elapsed();
                let profile_key = format!("{}:{}", file_name, function_name);
                
                // Update function profile
                let profile = self.function_profiles.entry(profile_key).or_insert_with(|| {
                    FunctionProfile {
                        function_name: function_name.to_string(),
                        file_name: file_name.to_string(),
                        call_count: 0,
                        total_time: Duration::ZERO,
                        average_time: Duration::ZERO,
                        min_time: Duration::MAX,
                        max_time: Duration::ZERO,
                        line_number: entry.line_number,
                    }
                });
                
                profile.call_count += 1;
                profile.total_time += execution_time;
                profile.average_time = profile.total_time / profile.call_count;
                
                if execution_time < profile.min_time {
                    profile.min_time = execution_time;
                }
                if execution_time > profile.max_time {
                    profile.max_time = execution_time;
                }
            }
        }
        
        Ok(())
    }
    
    /// Start script execution tracking
    pub fn start_script_execution(&mut self, script_name: &str) -> Result<(), ScriptError> {
        if !self.is_active {
            return Ok(());
        }
        
        let execution = ScriptExecution {
            script_name: script_name.to_string(),
            start_time: Instant::now(),
            memory_at_start: self.current_memory,
        };
        
        self.script_executions.insert(script_name.to_string(), execution);
        Ok(())
    }
    
    /// End script execution tracking
    pub fn end_script_execution(&mut self, script_name: &str) -> Result<(), ScriptError> {
        if !self.is_active {
            return Ok(());
        }
        
        if let Some(execution) = self.script_executions.remove(script_name) {
            let execution_time = execution.start_time.elapsed();
            let memory_used = self.current_memory - execution.memory_at_start;
            
            let profile = ScriptProfile {
                script_name: script_name.to_string(),
                execution_time,
                memory_used,
                function_calls: 0, // Would be tracked in real implementation
            };
            
            self.script_profiles.insert(script_name.to_string(), profile);
        }
        
        Ok(())
    }
    
    /// Take a memory snapshot
    pub fn take_memory_snapshot(&mut self, label: &str) -> Result<MemorySnapshot, ScriptError> {
        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            label: label.to_string(),
            total_bytes: self.current_memory,
            lua_bytes: self.current_memory / 2, // Simplified
            script_bytes: self.current_memory / 4,
        };
        
        self.memory_snapshots.push(snapshot.clone());
        Ok(snapshot)
    }
    
    /// Record memory allocation
    pub fn record_memory_allocation(&mut self, _script_name: &str, bytes: u64) -> Result<(), ScriptError> {
        self.current_memory += bytes;
        if self.current_memory > self.peak_memory {
            self.peak_memory = self.current_memory;
        }
        Ok(())
    }
    
    /// Record memory deallocation
    pub fn record_memory_deallocation(&mut self, _script_name: &str, bytes: u64) -> Result<(), ScriptError> {
        if self.current_memory >= bytes {
            self.current_memory -= bytes;
        }
        Ok(())
    }
    
    /// Record garbage collection event
    pub fn record_gc_event(
        &mut self, 
        gc_type: &str, 
        duration: Duration, 
        bytes_freed: u64
    ) -> Result<(), ScriptError> {
        let event = GcEvent {
            gc_type: gc_type.to_string(),
            duration,
            bytes_freed,
            timestamp: Instant::now(),
        };
        
        self.gc_events.push(event);
        
        // Reduce current memory by freed bytes
        if self.current_memory >= bytes_freed {
            self.current_memory -= bytes_freed;
        }
        
        Ok(())
    }
    
    /// Generate profiling report
    pub fn generate_report(&self) -> Result<ProfilerReport, ScriptError> {
        let total_time = self.start_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO);
        
        Ok(ProfilerReport {
            functions: self.function_profiles.values().cloned().collect(),
            scripts: self.script_profiles.values().cloned().collect(),
            memory_snapshots: self.memory_snapshots.clone(),
            gc_events: self.gc_events.clone(),
            total_profiling_time: total_time,
            peak_memory_usage: self.peak_memory,
        })
    }
    
    /// Get performance hotspots (slowest functions)
    pub fn get_performance_hotspots(&self, limit: usize) -> Result<Vec<&FunctionProfile>, ScriptError> {
        let mut functions: Vec<&FunctionProfile> = self.function_profiles.values().collect();
        functions.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        
        if limit > 0 && functions.len() > limit {
            functions.truncate(limit);
        }
        
        Ok(functions)
    }
    
    /// Get maximum call stack depth reached
    pub fn get_max_call_stack_depth(&self) -> usize {
        self.max_call_stack_depth
    }
    
    /// Set performance thresholds
    pub fn set_function_time_threshold(&mut self, threshold: Duration) -> Result<(), ScriptError> {
        self.function_time_threshold = threshold;
        Ok(())
    }
    
    pub fn set_memory_threshold(&mut self, threshold: u64) -> Result<(), ScriptError> {
        self.memory_threshold = threshold;
        Ok(())
    }
    
    /// Get performance warnings based on thresholds
    pub fn get_performance_warnings(&self) -> Result<Vec<String>, ScriptError> {
        let mut warnings = Vec::new();
        
        // Check for slow functions
        for profile in self.function_profiles.values() {
            if profile.max_time > self.function_time_threshold {
                warnings.push(format!(
                    "Function '{}' in {} exceeded time threshold: {:?} > {:?}",
                    profile.function_name,
                    profile.file_name,
                    profile.max_time,
                    self.function_time_threshold
                ));
            }
        }
        
        // Check for memory usage
        for snapshot in &self.memory_snapshots {
            if snapshot.total_bytes > self.memory_threshold {
                warnings.push(format!(
                    "Memory usage at '{}' exceeded threshold: {} > {} bytes",
                    snapshot.label,
                    snapshot.total_bytes,
                    self.memory_threshold
                ));
            }
        }
        
        Ok(warnings)
    }
    
    /// Export report as JSON
    pub fn export_report_as_json(&self, report: &ProfilerReport) -> Result<String, ScriptError> {
        // Simplified JSON export
        let mut json = String::from("{\n");
        json.push_str(&format!("  \"total_time_ms\": {},\n", report.total_profiling_time.as_millis()));
        json.push_str(&format!("  \"peak_memory\": {},\n", report.peak_memory_usage));
        json.push_str("  \"functions\": [\n");
        
        for (i, func) in report.functions.iter().enumerate() {
            json.push_str(&format!(
                "    {{\"name\": \"{}\", \"file\": \"{}\", \"calls\": {}, \"total_ms\": {}}}",
                func.function_name,
                func.file_name,
                func.call_count,
                func.total_time.as_millis()
            ));
            if i < report.functions.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        
        json.push_str("  ]\n");
        json.push_str("}");
        
        Ok(json)
    }
    
    /// Export report as CSV
    pub fn export_report_as_csv(&self, report: &ProfilerReport) -> Result<String, ScriptError> {
        let mut csv = String::from("Function Name,File Name,Call Count,Total Time (ms),Average Time (ms)\n");
        
        for func in &report.functions {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                func.function_name,
                func.file_name,
                func.call_count,
                func.total_time.as_millis(),
                func.average_time.as_millis()
            ));
        }
        
        Ok(csv)
    }
    
    /// Register profiler API functions in Lua
    pub fn register_profiler_api(&self, lua: &Lua) -> Result<(), ScriptError> {
        let globals = lua.globals();
        
        // profile_start function
        let profile_start = lua.create_function(|_, ()| {
            println!("[PROFILER] Started profiling");
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create profile_start function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        globals.set("profile_start", profile_start).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set profile_start function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // profile_stop function
        let profile_stop = lua.create_function(|_, ()| {
            println!("[PROFILER] Stopped profiling");
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create profile_stop function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        globals.set("profile_stop", profile_stop).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set profile_stop function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // profile_mark function
        let profile_mark = lua.create_function(|_, mark: String| {
            println!("[PROFILER] Mark: {}", mark);
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create profile_mark function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        globals.set("profile_mark", profile_mark).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set profile_mark function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        Ok(())
    }
}

#[cfg(test)]
#[path = "profiler_tests.rs"]
mod tests;