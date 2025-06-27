//! Tests for Lua script performance profiling system

#[cfg(test)]
mod tests {
    use crate::lua::profiler::{
        LuaProfiler, ProfilerReport, FunctionProfile, ScriptProfile, MemorySnapshot
    };
    use crate::ScriptError;
    use mlua::{Lua, Function as LuaFunction};
    use std::time::{Duration, Instant};
    
    #[test]
    fn test_profiler_creation() {
        let lua = Lua::new();
        let profiler = LuaProfiler::new(&lua).unwrap();
        
        assert!(!profiler.is_active());
        assert_eq!(profiler.get_function_count(), 0);
        assert!(profiler.get_memory_usage() >= 0);
    }
    
    #[test]
    fn test_profiler_activation() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        // Initially inactive
        assert!(!profiler.is_active());
        
        // Start profiling
        profiler.start_profiling().unwrap();
        assert!(profiler.is_active());
        
        // Stop profiling
        profiler.stop_profiling().unwrap();
        assert!(!profiler.is_active());
        
        // Reset profiler
        profiler.reset().unwrap();
        assert_eq!(profiler.get_function_count(), 0);
    }
    
    #[test]
    fn test_function_timing() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        profiler.start_profiling().unwrap();
        
        // Record function calls
        profiler.record_function_enter("test_script.lua", "calculate", 42).unwrap();
        std::thread::sleep(Duration::from_millis(10)); // Simulate work
        profiler.record_function_exit("test_script.lua", "calculate", 42).unwrap();
        
        profiler.record_function_enter("test_script.lua", "helper", 15).unwrap();
        std::thread::sleep(Duration::from_millis(5)); // Simulate work
        profiler.record_function_exit("test_script.lua", "helper", 15).unwrap();
        
        profiler.stop_profiling().unwrap();
        
        // Generate report
        let report = profiler.generate_report().unwrap();
        assert_eq!(report.functions.len(), 2);
        
        // Check timing data
        let calculate_profile = report.functions.iter()
            .find(|p| p.function_name == "calculate")
            .unwrap();
        assert_eq!(calculate_profile.call_count, 1);
        assert!(calculate_profile.total_time.as_millis() >= 10);
        
        let helper_profile = report.functions.iter()
            .find(|p| p.function_name == "helper")
            .unwrap();
        assert_eq!(helper_profile.call_count, 1);
        assert!(helper_profile.total_time.as_millis() >= 5);
    }
    
    #[test]
    fn test_memory_tracking() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        profiler.start_profiling().unwrap();
        
        // Take memory snapshots
        let initial_memory = profiler.take_memory_snapshot("initial").unwrap();
        assert!(initial_memory.total_bytes > 0);
        
        // Simulate memory allocation by creating objects
        profiler.record_memory_allocation("test_script.lua", 1024).unwrap();
        profiler.record_memory_allocation("test_script.lua", 512).unwrap();
        
        let after_alloc = profiler.take_memory_snapshot("after_allocation").unwrap();
        assert!(after_alloc.total_bytes > initial_memory.total_bytes);
        
        // Record memory deallocation
        profiler.record_memory_deallocation("test_script.lua", 512).unwrap();
        
        let after_dealloc = profiler.take_memory_snapshot("after_deallocation").unwrap();
        assert!(after_dealloc.total_bytes < after_alloc.total_bytes);
        
        profiler.stop_profiling().unwrap();
        
        // Check memory snapshots in report
        let report = profiler.generate_report().unwrap();
        assert_eq!(report.memory_snapshots.len(), 3);
    }
    
    #[test]
    fn test_script_level_profiling() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        profiler.start_profiling().unwrap();
        
        // Profile multiple scripts
        profiler.start_script_execution("main.lua").unwrap();
        std::thread::sleep(Duration::from_millis(15));
        profiler.end_script_execution("main.lua").unwrap();
        
        profiler.start_script_execution("utils.lua").unwrap();
        std::thread::sleep(Duration::from_millis(8));
        profiler.end_script_execution("utils.lua").unwrap();
        
        profiler.stop_profiling().unwrap();
        
        let report = profiler.generate_report().unwrap();
        assert_eq!(report.scripts.len(), 2);
        
        let main_script = report.scripts.iter()
            .find(|s| s.script_name == "main.lua")
            .unwrap();
        assert!(main_script.execution_time.as_millis() >= 15);
        
        let utils_script = report.scripts.iter()
            .find(|s| s.script_name == "utils.lua")
            .unwrap();
        assert!(utils_script.execution_time.as_millis() >= 8);
    }
    
    #[test]
    fn test_nested_function_calls() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        profiler.start_profiling().unwrap();
        
        // Simulate nested function calls
        profiler.record_function_enter("test.lua", "outer", 1).unwrap();
        profiler.record_function_enter("test.lua", "middle", 5).unwrap();
        profiler.record_function_enter("test.lua", "inner", 10).unwrap();
        
        std::thread::sleep(Duration::from_millis(5));
        
        profiler.record_function_exit("test.lua", "inner", 10).unwrap();
        profiler.record_function_exit("test.lua", "middle", 5).unwrap();
        profiler.record_function_exit("test.lua", "outer", 1).unwrap();
        
        profiler.stop_profiling().unwrap();
        
        let report = profiler.generate_report().unwrap();
        
        // All functions should be recorded
        assert_eq!(report.functions.len(), 3);
        assert!(report.functions.iter().any(|f| f.function_name == "outer"));
        assert!(report.functions.iter().any(|f| f.function_name == "middle"));
        assert!(report.functions.iter().any(|f| f.function_name == "inner"));
        
        // Check call stack depth tracking
        assert_eq!(profiler.get_max_call_stack_depth(), 3);
    }
    
    #[test]
    fn test_performance_hotspots() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        profiler.start_profiling().unwrap();
        
        // Simulate hotspot - function called many times
        for i in 0..100 {
            profiler.record_function_enter("hotspot.lua", "frequently_called", i).unwrap();
            if i % 10 == 0 {
                std::thread::sleep(Duration::from_millis(1)); // Some calls are slower
            }
            profiler.record_function_exit("hotspot.lua", "frequently_called", i).unwrap();
        }
        
        // Simulate slower function called once
        profiler.record_function_enter("slow.lua", "slow_function", 1).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        profiler.record_function_exit("slow.lua", "slow_function", 1).unwrap();
        
        profiler.stop_profiling().unwrap();
        
        let report = profiler.generate_report().unwrap();
        let hotspots = profiler.get_performance_hotspots(5).unwrap();
        
        // Should identify the slow function as a hotspot
        assert!(!hotspots.is_empty());
        assert!(hotspots.iter().any(|h| h.function_name == "slow_function"));
        
        // Frequently called function should also be identified
        let frequent_fn = report.functions.iter()
            .find(|f| f.function_name == "frequently_called")
            .unwrap();
        assert_eq!(frequent_fn.call_count, 100);
    }
    
    #[test]
    fn test_gc_tracking() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        profiler.start_profiling().unwrap();
        
        // Simulate garbage collection events
        profiler.record_gc_event("minor", Duration::from_millis(5), 1024).unwrap();
        profiler.record_gc_event("major", Duration::from_millis(25), 8192).unwrap();
        profiler.record_gc_event("minor", Duration::from_millis(3), 512).unwrap();
        
        profiler.stop_profiling().unwrap();
        
        let report = profiler.generate_report().unwrap();
        assert_eq!(report.gc_events.len(), 3);
        
        let total_gc_time: Duration = report.gc_events.iter()
            .map(|gc| gc.duration)
            .sum();
        assert!(total_gc_time.as_millis() >= 33);
        
        let total_freed: u64 = report.gc_events.iter()
            .map(|gc| gc.bytes_freed)
            .sum();
        assert_eq!(total_freed, 1024 + 8192 + 512);
    }
    
    #[test]
    fn test_performance_thresholds() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        // Set performance thresholds
        profiler.set_function_time_threshold(Duration::from_millis(10)).unwrap();
        profiler.set_memory_threshold(1024).unwrap();
        
        profiler.start_profiling().unwrap();
        
        // Fast function - should not trigger threshold
        profiler.record_function_enter("fast.lua", "fast_fn", 1).unwrap();
        std::thread::sleep(Duration::from_millis(2));
        profiler.record_function_exit("fast.lua", "fast_fn", 1).unwrap();
        
        // Slow function - should trigger threshold
        profiler.record_function_enter("slow.lua", "slow_fn", 1).unwrap();
        std::thread::sleep(Duration::from_millis(15));
        profiler.record_function_exit("slow.lua", "slow_fn", 1).unwrap();
        
        // Large memory allocation - should trigger threshold
        profiler.record_memory_allocation("memory_heavy.lua", 2048).unwrap();
        
        profiler.stop_profiling().unwrap();
        
        let warnings = profiler.get_performance_warnings().unwrap();
        assert!(!warnings.is_empty());
        
        // Should have warnings for slow function and large allocation
        assert!(warnings.iter().any(|w| w.contains("slow_fn")));
        assert!(warnings.iter().any(|w| w.contains("memory_heavy.lua")));
        assert!(!warnings.iter().any(|w| w.contains("fast_fn")));
    }
    
    #[test]
    fn test_profiler_report_serialization() {
        let lua = Lua::new();
        let mut profiler = LuaProfiler::new(&lua).unwrap();
        
        profiler.start_profiling().unwrap();
        
        // Add some profile data
        profiler.record_function_enter("test.lua", "func1", 1).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        profiler.record_function_exit("test.lua", "func1", 1).unwrap();
        
        profiler.stop_profiling().unwrap();
        
        let report = profiler.generate_report().unwrap();
        
        // Test JSON serialization
        let json_output = profiler.export_report_as_json(&report).unwrap();
        assert!(json_output.contains("func1"));
        assert!(json_output.contains("test.lua"));
        
        // Test CSV export
        let csv_output = profiler.export_report_as_csv(&report).unwrap();
        assert!(csv_output.contains("Function Name"));
        assert!(csv_output.contains("func1"));
    }
    
    #[test]
    fn test_profiler_api_registration() {
        let lua = Lua::new();
        let profiler = LuaProfiler::new(&lua).unwrap();
        
        // Register profiler API
        profiler.register_profiler_api(&lua).unwrap();
        
        // Check that profiler functions are available
        let globals = lua.globals();
        assert!(globals.get::<_, LuaFunction>("profile_start").is_ok());
        assert!(globals.get::<_, LuaFunction>("profile_stop").is_ok());
        assert!(globals.get::<_, LuaFunction>("profile_mark").is_ok());
        
        // Test profiler functions from Lua
        lua.load(r#"
            profile_start()
            profile_mark("test_checkpoint")
            profile_stop()
        "#).exec().unwrap();
    }
}