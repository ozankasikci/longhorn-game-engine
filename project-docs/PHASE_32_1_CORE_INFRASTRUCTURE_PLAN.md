# PHASE 32.1: Core Infrastructure Implementation Plan

## Overview

This document details the implementation plan for Phase 1 of the TypeScript migration: building the core infrastructure for TypeScript script execution within the Longhorn Game Engine.

## Phase Objectives

### Primary Goals
1. Implement basic TypeScript runtime that satisfies the `ScriptRuntime` trait
2. Create V8 isolate management and sandboxing system
3. Port security and resource limit enforcement
4. Establish TypeScript compilation pipeline
5. Implement basic error handling with source map support

### Success Criteria
- [ ] TypeScript scripts can be loaded and executed
- [ ] Security sandbox prevents unauthorized access
- [ ] Resource limits are enforced (memory, execution time)
- [ ] Compilation errors provide accurate line/column information
- [ ] Hot reloading works for simple scripts
- [ ] Performance within 30% of Lua baseline (initial target)

## Technical Architecture

### Core Components

#### 1. TypeScript Engine (`src/typescript/engine.rs`)

```rust
pub struct TypeScriptEngine {
    /// V8 JavaScript engine isolate
    isolate: v8::OwnedIsolate,
    
    /// Global execution context
    global_context: v8::Global<v8::Context>,
    
    /// TypeScript to JavaScript compiler
    compiler: TypeScriptCompiler,
    
    /// Permission system for API access
    permissions: ScriptCapabilities,
    
    /// Resource usage tracking
    resource_tracker: ResourceTracker,
    
    /// Loaded and compiled scripts
    loaded_scripts: HashMap<ScriptId, CompiledScript>,
    
    /// Error context enrichment
    error_enricher: ErrorEnricher,
}

#[derive(Debug)]
pub struct CompiledScript {
    /// Compiled JavaScript code
    js_code: String,
    
    /// Source map for debugging
    source_map: Option<String>,
    
    /// Original TypeScript source
    ts_source: String,
    
    /// Script metadata
    metadata: ScriptMetadata,
    
    /// Compilation timestamp
    compiled_at: SystemTime,
}
```

#### 2. TypeScript Compiler (`src/typescript/compiler.rs`)

```rust
pub struct TypeScriptCompiler {
    /// SWC compiler instance
    swc_compiler: Arc<swc::Compiler>,
    
    /// Compilation options
    options: TypeScriptOptions,
    
    /// Source map cache
    source_map_cache: HashMap<String, String>,
    
    /// Type checking mode
    type_check_mode: TypeCheckMode,
}

#[derive(Debug, Clone)]
pub struct TypeScriptOptions {
    /// Target ECMAScript version
    pub target: EcmaVersion,
    
    /// Module system (ES6, CommonJS)
    pub module: ModuleConfig,
    
    /// Enable strict mode
    pub strict: bool,
    
    /// Generate source maps
    pub source_map: bool,
    
    /// Minify output
    pub minify: bool,
}

pub enum TypeCheckMode {
    /// No type checking (fast compilation)
    None,
    
    /// Basic type checking
    Basic,
    
    /// Full TypeScript type checking
    Strict,
}
```

#### 3. Security and Sandboxing (`src/typescript/security.rs`)

```rust
pub struct SecurityManager {
    /// Allowed global objects
    allowed_globals: HashSet<String>,
    
    /// API permission mappings
    permission_mappings: HashMap<String, ApiPermission>,
    
    /// Import allowlist
    allowed_imports: HashSet<String>,
    
    /// Execution context isolation
    context_isolator: ContextIsolator,
}

pub struct ContextIsolator {
    /// V8 isolate pool for script execution
    isolate_pool: IsolatePool,
    
    /// Context creation parameters
    context_params: ContextParameters,
    
    /// Security callback handlers
    security_callbacks: SecurityCallbacks,
}

pub trait SecurityPolicy {
    fn can_access_api(&self, api_name: &str, permissions: &ScriptCapabilities) -> bool;
    fn can_import_module(&self, module_path: &str) -> bool;
    fn validate_global_access(&self, property: &str) -> bool;
}
```

#### 4. Resource Management (`src/typescript/resources.rs`)

```rust
pub struct ResourceTracker {
    /// Memory usage tracking
    memory_tracker: MemoryTracker,
    
    /// Execution time limits
    execution_timer: ExecutionTimer,
    
    /// Call stack depth tracking
    stack_tracker: StackTracker,
    
    /// Resource limits configuration
    limits: ResourceLimits,
}

pub struct MemoryTracker {
    /// Current memory usage
    current_usage: AtomicUsize,
    
    /// Peak memory usage
    peak_usage: AtomicUsize,
    
    /// V8 heap statistics
    heap_stats: V8HeapStats,
}

pub struct ExecutionTimer {
    /// Script start time
    start_time: Option<Instant>,
    
    /// Maximum execution duration
    max_duration: Duration,
    
    /// Timeout callback
    timeout_handler: TimeoutHandler,
}
```

## Implementation Details

### 1. V8 Integration Setup

```rust
// Initialize V8 platform
fn initialize_v8_platform() -> Result<(), TypeScriptError> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
    Ok(())
}

// Create isolated execution context
fn create_isolated_context(
    isolate: &mut v8::Isolate,
    permissions: &ScriptCapabilities,
) -> Result<v8::Local<v8::Context>, TypeScriptError> {
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    
    // Set up restricted global object
    setup_restricted_globals(scope, permissions)?;
    
    // Install security callbacks
    install_security_callbacks(scope)?;
    
    Ok(context)
}

// Restricted global setup
fn setup_restricted_globals(
    scope: &mut v8::ContextScope,
    permissions: &ScriptCapabilities,
) -> Result<(), TypeScriptError> {
    let global = scope.get_current_context().global(scope);
    
    // Remove dangerous globals
    remove_dangerous_globals(scope, global)?;
    
    // Add permitted APIs based on capabilities
    if permissions.has_capability("console_write") {
        install_console_api(scope, global)?;
    }
    
    if permissions.has_capability("math_api") {
        install_math_api(scope, global)?;
    }
    
    Ok(())
}
```

### 2. TypeScript Compilation Pipeline

```rust
impl TypeScriptCompiler {
    pub fn compile_script(
        &mut self,
        source: &str,
        path: &str,
    ) -> Result<CompiledScript, CompilationError> {
        // Parse TypeScript with SWC
        let parsed = self.parse_typescript(source, path)?;
        
        // Transform to JavaScript
        let js_result = self.transform_to_javascript(parsed)?;
        
        // Generate source map
        let source_map = if self.options.source_map {
            Some(self.generate_source_map(&js_result)?)
        } else {
            None
        };
        
        // Perform type checking if enabled
        if matches!(self.type_check_mode, TypeCheckMode::Basic | TypeCheckMode::Strict) {
            self.type_check_source(source, path)?;
        }
        
        Ok(CompiledScript {
            js_code: js_result.code,
            source_map,
            ts_source: source.to_string(),
            metadata: ScriptMetadata::from_path(path),
            compiled_at: SystemTime::now(),
        })
    }
    
    fn parse_typescript(
        &self,
        source: &str,
        path: &str,
    ) -> Result<swc_ecma_ast::Module, CompilationError> {
        let source_file = SourceFile::new(
            FileName::Real(PathBuf::from(path)),
            false,
            FileName::Real(PathBuf::from(path)),
            source.to_string(),
        );
        
        let syntax = Syntax::Typescript(TsConfig {
            tsx: path.ends_with(".tsx"),
            decorators: true,
            dts: path.ends_with(".d.ts"),
            no_early_errors: false,
            disallow_ambiguous_jsx_like: true,
        });
        
        self.swc_compiler
            .parse_js(source_file, &syntax, EsVersion::Es2022, None, &mut vec![])
            .map_err(CompilationError::ParseError)
    }
}
```

### 3. Script Runtime Implementation

```rust
impl ScriptRuntime for TypeScriptEngine {
    fn initialize(&mut self) -> ScriptResult<()> {
        // Initialize V8 if not already done
        if !v8::V8::is_initialized() {
            initialize_v8_platform()?;
        }
        
        // Create isolate with resource limits
        let create_params = v8::CreateParams::default()
            .heap_limits(
                self.resource_tracker.limits.initial_heap_size,
                self.resource_tracker.limits.max_heap_size,
            );
            
        self.isolate = v8::Isolate::new(create_params);
        
        // Set up global context
        let context = create_isolated_context(&mut self.isolate, &self.permissions)?;
        self.global_context = v8::Global::new(&mut self.isolate, context);
        
        // Initialize compiler
        self.compiler.initialize()?;
        
        Ok(())
    }
    
    fn load_script(&mut self, metadata: ScriptMetadata, source: &str) -> ScriptResult<()> {
        // Compile TypeScript to JavaScript
        let compiled = self.compiler.compile_script(source, &metadata.path)
            .map_err(|e| ScriptError::CompilationError {
                script_name: metadata.path.clone(),
                error: e.to_string(),
            })?;
        
        // Store compiled script
        self.loaded_scripts.insert(metadata.id, compiled);
        
        Ok(())
    }
    
    fn execute_script(&mut self, id: ScriptId) -> ScriptResult<()> {
        let script = self.loaded_scripts.get(&id)
            .ok_or_else(|| ScriptError::NotFound(format!("Script ID: {}", id.0)))?;
        
        // Create execution scope
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Local::new(scope, &self.global_context);
        let scope = &mut v8::ContextScope::new(scope, context);
        
        // Start resource tracking
        self.resource_tracker.start_execution()?;
        
        // Execute JavaScript
        let result = self.execute_in_scope(scope, &script.js_code, &script.metadata.path);
        
        // Stop resource tracking
        self.resource_tracker.stop_execution()?;
        
        result
    }
    
    fn supports_type(&self, script_type: &ScriptType) -> bool {
        matches!(script_type, ScriptType::TypeScript | ScriptType::JavaScript)
    }
    
    fn update(&mut self, delta_time: f32) -> ScriptResult<()> {
        // Update resource tracking
        self.resource_tracker.update()?;
        
        // Check for memory pressure
        if self.resource_tracker.is_memory_pressure() {
            self.perform_garbage_collection()?;
        }
        
        // Update compiler cache
        self.compiler.update_cache()?;
        
        Ok(())
    }
}
```

## Implementation Phases

### Week 1: Basic V8 Integration
**Days 1-2:**
- Set up V8 dependencies and build configuration
- Implement basic isolate creation and management
- Create minimal JavaScript execution capability

**Days 3-5:**
- Implement security sandbox with restricted globals
- Add basic error handling and reporting
- Create initial test suite

### Week 2: TypeScript Compilation
**Days 1-3:**
- Integrate SWC TypeScript compiler
- Implement compilation pipeline with error handling
- Add source map generation

**Days 4-5:**
- Implement compilation caching
- Add type checking integration
- Create compilation performance benchmarks

### Week 3: Resource Management
**Days 1-2:**
- Port resource limit system from Lua
- Implement V8 heap monitoring
- Add execution timeout handling

**Days 3-5:**
- Integrate with existing permission system
- Implement API access controls
- Add resource usage reporting

### Week 4: ScriptRuntime Integration
**Days 1-3:**
- Implement complete ScriptRuntime trait
- Integrate with existing ScriptManager
- Add script loading and execution

**Days 4-5:**
- Implement hot reloading capability
- Add comprehensive error enrichment
- Performance testing and optimization

### Week 5: Testing and Polish
**Days 1-3:**
- Comprehensive unit and integration tests
- Performance benchmarking against Lua
- Memory leak detection and fixes

**Days 4-5:**
- Documentation and code review
- Bug fixes and edge case handling
- Preparation for Phase 2

## Dependencies

### External Crates
```toml
[dependencies]
v8 = "0.84"
swc = "0.275"
swc_ecma_parser = "0.141"
swc_ecma_transforms = "0.229"
swc_ecma_codegen = "0.147"
tokio = "1.0"  # For async compilation
serde_json = "1.0"  # For source maps
```

### Internal Dependencies
- `engine-ecs-core`: Entity component system
- `engine-scripting`: Existing scripting infrastructure
- Existing security and resource management systems

## Testing Strategy

### Unit Tests
- V8 isolate management
- TypeScript compilation pipeline
- Security sandbox functionality
- Resource limit enforcement

### Integration Tests
- ScriptRuntime trait compliance
- ScriptManager integration
- Hot reloading scenarios
- Error handling and reporting

### Performance Tests
- Compilation speed benchmarks
- Execution performance vs Lua
- Memory usage analysis
- Startup time measurement

## Acceptance Criteria

### Functional Requirements
- [ ] Load and execute simple TypeScript scripts
- [ ] Enforce all security restrictions from Lua system
- [ ] Provide accurate error messages with source locations
- [ ] Support hot reloading without memory leaks
- [ ] Integrate with existing ScriptManager API

### Performance Requirements
- [ ] Compilation time under 500ms for typical scripts
- [ ] Execution performance within 30% of Lua baseline
- [ ] Memory usage increase under 100% of Lua baseline
- [ ] Hot reload time under 1 second

### Quality Requirements
- [ ] 100% test coverage for security-critical code
- [ ] Zero unsafe Rust code in public APIs
- [ ] Comprehensive error handling and recovery
- [ ] Memory safety validation with Valgrind/MIRI

## Risk Mitigation

### Technical Risks
1. **V8 Integration Complexity**
   - *Mitigation*: Start with minimal integration, expand gradually
   - *Fallback*: QuickJS alternative prepared

2. **Performance Concerns**
   - *Mitigation*: Continuous benchmarking, optimization checkpoints
   - *Fallback*: Hybrid Lua/TypeScript approach

3. **Memory Management**
   - *Mitigation*: Careful isolate lifecycle management
   - *Fallback*: Aggressive GC tuning, isolate pooling

### Timeline Risks
1. **V8 Learning Curve**
   - *Mitigation*: Dedicated V8 expert consultation
   - *Buffer*: 1-week contingency built into schedule

2. **Security Implementation**
   - *Mitigation*: Reuse existing patterns from Lua
   - *Buffer*: Security review checkpoint before Phase 2

## Next Steps

1. **Week 1 Sprint Planning**
   - Set up development environment
   - Create detailed task breakdown
   - Begin V8 integration spike

2. **Regular Checkpoints**
   - Daily standups for first 2 weeks
   - Weekly milestone reviews
   - Performance benchmark reviews

3. **Stakeholder Communication**
   - Weekly progress reports
   - Security review at midpoint
   - Performance review before Phase 2

---

**Document Version**: 1.0  
**Last Updated**: 2025-06-27  
**Next Review**: 2025-07-04  
**Approval Required**: Architecture Team  