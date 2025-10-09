# Phase 30: Scripting Memory & Performance Improvements

## Overview
Phase 30 focuses on memory management issues and performance optimizations in the scripting system. This phase addresses resource leaks, inefficient operations, and memory usage problems identified in the analysis.

## Previous Phase Context
Following Phase 29's security and architecture fixes, Phase 30 optimizes the now-secure scripting system for production performance and resource management.

## Goals
1. **Fix memory management issues** - Eliminate leaks and unbounded growth
2. **Implement performance optimizations** - Add caching and efficient queries
3. **Add proper resource management** - Control Lua garbage collection
4. **Optimize hot paths** - Reduce allocations and improve batching

## Current Problems

### Memory Management Issues
- **Unbounded growth**: Console messages and script instances not cleaned up
- **No GC management**: Lua garbage collection not explicitly controlled
- **Resource leaks**: Hot reload accumulates garbage over time
- **Memory inefficiency**: No object pooling, excessive allocations

### Performance Concerns
- **Scripts recompiled every load**: No bytecode caching mechanism
- **Inefficient queries**: Entity searches on every frame
- **Hot reload polling**: File system checked every frame instead of using watchers
- **Poor batching**: No script execution optimization

### Files to Modify
- `crates/implementation/engine-scripting/src/manager.rs`
- `crates/implementation/engine-scripting/src/file_manager.rs`
- `crates/implementation/engine-scripting/src/runtime.rs`
- `crates/implementation/engine-scripting/src/lua_script_system.rs`

## Implementation Tasks

### Task 1: Memory Management Fixes (Priority: High)

#### 1.1 Implement Proper Cleanup
**Location**: `manager.rs`
```rust
pub struct ScriptManager {
    scripts: HashMap<ScriptId, LoadedScript>,
    cleanup_scheduler: CleanupScheduler,
}

impl ScriptManager {
    pub fn cleanup_unused_scripts(&mut self) {
        // Remove scripts with no entity bindings
        // Clean up orphaned resources
        // Force Lua GC when appropriate
    }
}
```

#### 1.2 Lua Garbage Collection Management
**Location**: `runtime.rs`
```rust
pub struct LuaRuntime {
    lua: Lua,
    gc_threshold: usize,
    last_gc: Instant,
}

impl LuaRuntime {
    pub fn manage_gc(&mut self) {
        let memory_usage = self.lua.gc_get_count();
        if memory_usage > self.gc_threshold || 
           self.last_gc.elapsed() > Duration::from_secs(30) {
            self.lua.gc_collect()?;
            self.last_gc = Instant::now();
        }
    }
}
```

#### 1.3 Resource Pooling
**New file**: `resource_pool.rs`
```rust
pub struct ScriptResourcePool {
    lua_values: Vec<Value>,
    string_cache: HashMap<String, mlua::String>,
    table_pool: Vec<Table>,
}
```

### Task 2: Bytecode Caching (Priority: High)

#### 2.1 Bytecode Cache System
**New file**: `bytecode_cache.rs`
```rust
pub struct BytecodeCache {
    cache: HashMap<PathBuf, CachedBytecode>,
    max_size: usize,
}

pub struct CachedBytecode {
    bytecode: Vec<u8>,
    last_modified: SystemTime,
    source_hash: u64,
}
```

#### 2.2 Smart Recompilation
**Location**: `manager.rs`
```rust
impl ScriptManager {
    pub fn load_script_cached(&mut self, path: &Path) -> Result<ScriptId, ScriptError> {
        if let Some(cached) = self.bytecode_cache.get(path) {
            if cached.is_valid(path) {
                return self.load_from_bytecode(&cached.bytecode);
            }
        }
        
        // Compile and cache
        let source = std::fs::read_to_string(path)?;
        let bytecode = self.compile_to_bytecode(&source)?;
        self.bytecode_cache.insert(path, bytecode.clone());
        self.load_from_bytecode(&bytecode)
    }
}
```

### Task 3: Efficient Entity Queries (Priority: High)

#### 3.1 Query Caching System
**Location**: `lua_script_system.rs`
```rust
pub struct EntityQueryCache {
    cached_queries: HashMap<QuerySignature, Vec<EntityId>>,
    last_update: HashMap<QuerySignature, u64>,
    world_version: u64,
}

impl EntityQueryCache {
    pub fn get_entities_cached(&mut self, query: QuerySignature) -> &Vec<EntityId> {
        if self.is_cache_valid(&query) {
            &self.cached_queries[&query]
        } else {
            self.refresh_query(query)
        }
    }
}
```

#### 3.2 Batch Script Execution
**Location**: `lua_script_system.rs`
```rust
impl ScriptSystem {
    pub fn execute_scripts_batched(&mut self) {
        // Group scripts by execution type
        let mut update_scripts = Vec::new();
        let mut event_scripts = Vec::new();
        
        // Execute in batches for better cache locality
        self.execute_update_batch(&update_scripts);
        self.execute_event_batch(&event_scripts);
    }
}
```

### Task 4: File System Optimization (Priority: Medium)

#### 4.1 File Watching System
**Location**: `file_manager.rs`
```rust
use notify::{Watcher, RecommendedWatcher, RecursiveMode};

pub struct ScriptFileWatcher {
    watcher: RecommendedWatcher,
    changed_files: mpsc::Receiver<PathBuf>,
}

impl ScriptFileWatcher {
    pub fn new() -> Result<Self, ScriptError> {
        let (tx, rx) = mpsc::channel();
        let watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                if let Some(path) = event.paths.first() {
                    tx.send(path.clone()).ok();
                }
            }
        })?;
        
        Ok(Self {
            watcher,
            changed_files: rx,
        })
    }
    
    pub fn check_for_changes(&mut self) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        while let Ok(path) = self.changed_files.try_recv() {
            changed.push(path);
        }
        changed
    }
}
```

#### 4.2 Smart Hot Reload
**Location**: `file_manager.rs`
```rust
impl FileManager {
    pub fn hot_reload_changed(&mut self) -> Result<Vec<ScriptId>, ScriptError> {
        let changed_files = self.watcher.check_for_changes();
        let mut reloaded_scripts = Vec::new();
        
        for path in changed_files {
            if let Some(script_id) = self.path_to_script.get(&path) {
                self.reload_script_preserving_state(*script_id)?;
                reloaded_scripts.push(*script_id);
            }
        }
        
        Ok(reloaded_scripts)
    }
}
```

### Task 5: Memory Profiling and Monitoring (Priority: Medium)

#### 5.1 Memory Usage Tracking
**New file**: `memory_profiler.rs`
```rust
pub struct ScriptMemoryProfiler {
    script_memory: HashMap<ScriptId, usize>,
    total_lua_memory: usize,
    peak_memory: usize,
    gc_stats: GcStats,
}

impl ScriptMemoryProfiler {
    pub fn update_memory_stats(&mut self, lua: &Lua) {
        self.total_lua_memory = lua.gc_get_count();
        if self.total_lua_memory > self.peak_memory {
            self.peak_memory = self.total_lua_memory;
        }
    }
    
    pub fn get_memory_report(&self) -> MemoryReport {
        MemoryReport {
            total_memory: self.total_lua_memory,
            peak_memory: self.peak_memory,
            per_script: self.script_memory.clone(),
            gc_stats: self.gc_stats.clone(),
        }
    }
}
```

## Testing Requirements

### Memory Testing
1. **Memory leak tests** - Long-running script execution
2. **GC efficiency tests** - Memory usage patterns
3. **Hot reload stress tests** - Multiple reload cycles
4. **Resource pool tests** - Object reuse verification

### Performance Testing
1. **Bytecode caching benchmarks** - Compilation time savings
2. **Query caching benchmarks** - Entity query performance
3. **Batch execution benchmarks** - Script execution throughput
4. **File watching benchmarks** - Hot reload responsiveness

## Success Criteria

### Memory Management
- [ ] No memory leaks in long-running tests (24+ hours)
- [ ] Bounded memory growth during hot reload cycles
- [ ] Lua GC properly managed and tuned
- [ ] Resource pooling showing measurable memory reduction

### Performance
- [ ] 50%+ reduction in script compilation time (bytecode caching)
- [ ] 30%+ improvement in entity query performance
- [ ] File watching replaces polling (0% CPU when no changes)
- [ ] Batch execution shows improved throughput

### Monitoring
- [ ] Memory usage tracking and reporting
- [ ] Performance metrics collection
- [ ] Profiling tools integration

## Timeline
- **Week 1**: Memory management fixes and GC optimization
- **Week 2**: Bytecode caching and query optimization
- **Week 3**: File watching and performance monitoring

## Risks and Mitigation

### Risk: Memory Management Complexity
- **Mitigation**: Incremental implementation with extensive testing
- **Fallback**: Conservative GC settings, simpler resource management

### Risk: Caching Invalidation Bugs
- **Mitigation**: Comprehensive cache validation tests
- **Fallback**: Disable caching in debug builds

### Risk: Performance Regression
- **Mitigation**: Continuous benchmarking throughout development
- **Fallback**: Feature flags for performance optimizations

## Dependencies
- **Phase 29**: Must be completed (security and architecture fixes)
- **External**: `notify` crate for file watching

## Next Phase
Phase 31 will focus on API design improvements and feature completion once performance is optimized.

## Notes
- Memory management is critical for long-running applications
- Performance improvements must not compromise security from Phase 29
- All optimizations should be measurable and benchmarked