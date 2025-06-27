# PHASE 32: TypeScript Migration Plan

## Overview

This document outlines the comprehensive migration strategy from Lua to TypeScript for the Longhorn Game Engine scripting system. The migration will maintain existing security, performance, and architectural patterns while leveraging TypeScript's superior developer ecosystem and tooling.

## Current State Assessment

### Strengths of Current Lua Implementation
- ✅ Robust security and sandboxing
- ✅ Resource limit enforcement
- ✅ Permission-based API access
- ✅ Hot reloading capabilities
- ✅ ECS integration
- ✅ Clean runtime abstraction
- ✅ Comprehensive error handling

### Limitations of Lua
- ❌ Limited third-party ecosystem
- ❌ Basic IDE support compared to TypeScript
- ❌ No compile-time type checking
- ❌ Fewer developers familiar with Lua
- ❌ Limited integration with modern web services

## TypeScript Benefits

### Developer Experience
- **Static Type Checking**: Catch errors at compile time
- **Superior IDE Support**: IntelliSense, refactoring, debugging
- **Rich Ecosystem**: npm packages for all major services
- **Familiar Syntax**: More developers know TypeScript/JavaScript
- **Modern Language Features**: async/await, destructuring, modules

### Integration Advantages
- **Service SDKs**: Every major service provides TypeScript/JavaScript SDKs
- **Tool Ecosystem**: Bundlers, formatters, linters
- **Documentation**: Auto-generated docs from types
- **Testing**: Jest, Vitest, and other mature testing frameworks

## Technical Architecture

### JavaScript Engine Selection

**Recommendation: V8 via rusty_v8**

**Rationale:**
- Mature and battle-tested
- Excellent TypeScript support via compilation
- High performance with JIT compilation
- Rich debugging capabilities
- Used by Deno (proven Rust integration)

**Alternative: QuickJS**
- Smaller footprint
- Easier embedding
- Better for resource-constrained environments

### TypeScript Compilation Strategy

**Two-Phase Approach:**

1. **Development Mode**: Just-in-time TypeScript compilation
   - Use SWC for fast transpilation
   - Type checking during development
   - Source map preservation for debugging

2. **Production Mode**: Ahead-of-time compilation
   - Pre-compile all scripts to JavaScript
   - Bundle with tree-shaking for optimal size
   - Cached compilation for hot reloading

### Security Model Preservation

**V8 Isolate-Based Sandboxing:**
```rust
pub struct TypeScriptEngine {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
    permissions: ScriptCapabilities,
    resource_limits: ResourceLimits,
}
```

**Maintained Security Features:**
- Permission-based API access
- Resource limits (memory, execution time)
- Input validation and sanitization
- Global object restriction
- Import/require control

## Migration Strategy

### Phase 1: Core Infrastructure (4-6 weeks)

**Goals:**
- Implement basic TypeScript runtime
- Port security and resource systems
- Create minimal API bindings

**Deliverables:**
1. `TypeScriptEngine` implementing `ScriptRuntime` trait
2. V8 isolate management and sandboxing
3. Resource limit enforcement
4. Basic error handling and reporting
5. Simple TypeScript compilation pipeline

**Key Files to Create:**
- `src/typescript/mod.rs`
- `src/typescript/engine.rs`
- `src/typescript/compiler.rs`
- `src/typescript/security.rs`

### Phase 2: API Binding System (3-4 weeks)

**Goals:**
- Port all Lua API bindings to TypeScript
- Implement permission-based access
- Create TypeScript declaration files

**Deliverables:**
1. Complete engine API bindings
2. ECS component access system
3. Math, time, input, and physics APIs
4. Generated TypeScript declaration files
5. API documentation

**Key Components:**
- Engine globals injection
- Type-safe component access
- Event system bindings
- Asset loading APIs

### Phase 3: ECS Integration (2-3 weeks)

**Goals:**
- Deep ECS integration with TypeScript
- Component system wrapper
- Entity lifecycle management

**Deliverables:**
1. Entity wrapper classes
2. Component query system
3. Entity context management
4. Lifecycle method support (`init`, `update`, etc.)

### Phase 4: Development Experience (3-4 weeks)

**Goals:**
- Hot reloading with TypeScript compilation
- Debugging support with source maps
- IDE integration and tooling

**Deliverables:**
1. TypeScript hot reloading system
2. Source map preservation
3. Debug protocol integration
4. Project templates and examples
5. Development server for script editing

### Phase 5: Performance Optimization (2-3 weeks)

**Goals:**
- Optimize compilation and execution performance
- Implement caching strategies
- Bundle optimization

**Deliverables:**
1. Incremental TypeScript compilation
2. Script bundling and tree-shaking
3. V8 snapshot optimization
4. Performance monitoring and profiling

### Phase 6: Testing and Migration Tools (2-3 weeks)

**Goals:**
- Automated Lua to TypeScript conversion
- Comprehensive testing
- Migration utilities

**Deliverables:**
1. Lua-to-TypeScript transpilation tool
2. Test suite for TypeScript runtime
3. Performance benchmarking
4. Migration documentation

## Technical Implementation Details

### Runtime Architecture

```rust
// src/typescript/engine.rs
pub struct TypeScriptEngine {
    isolate: v8::OwnedIsolate,
    global_context: v8::Global<v8::Context>,
    compiler: TypeScriptCompiler,
    permissions: ScriptCapabilities,
    resource_tracker: ResourceTracker,
    loaded_scripts: HashMap<ScriptId, CompiledScript>,
}

impl ScriptRuntime for TypeScriptEngine {
    fn initialize(&mut self) -> ScriptResult<()> {
        // Initialize V8 isolate
        // Set up global context with restricted APIs
        // Install permission-based bindings
    }
    
    fn load_script(&mut self, metadata: ScriptMetadata, source: &str) -> ScriptResult<()> {
        // Compile TypeScript to JavaScript
        // Create script context with permissions
        // Cache compiled result
    }
    
    fn execute_script(&mut self, id: ScriptId) -> ScriptResult<()> {
        // Execute in isolated context
        // Enforce resource limits
        // Handle errors with source maps
    }
}
```

### TypeScript Compilation

```rust
// src/typescript/compiler.rs
pub struct TypeScriptCompiler {
    swc_compiler: swc::Compiler,
    type_checker: Option<TypeChecker>,
    source_map_cache: HashMap<String, String>,
}

impl TypeScriptCompiler {
    pub fn compile(&mut self, source: &str, path: &str) -> CompileResult<CompiledScript> {
        // Parse TypeScript with SWC
        // Generate source maps
        // Perform type checking if enabled
        // Return JavaScript + source map
    }
}
```

### API Binding System

```typescript
// Engine API type definitions
declare global {
    const Engine: {
        math: {
            vec3(x: number, y: number, z: number): Vector3;
            vec2(x: number, y: number): Vector2;
            lerp(a: number, b: number, t: number): number;
        };
        
        time: {
            readonly deltaTime: number;
            readonly totalTime: number;
            readonly frameCount: number;
        };
        
        input: {
            isKeyPressed(key: string): boolean;
            getMousePosition(): Vector2;
            isMouseButtonPressed(button: number): boolean;
        };
        
        world: {
            createEntity(components?: ComponentMap): Entity;
            removeEntity(entity: Entity): void;
            query<T extends Component>(type: ComponentType<T>): QueryResult<T>;
        };
        
        assets: {
            loadTexture(path: string): Promise<Texture>;
            loadSound(path: string): Promise<Sound>;
            loadMesh(path: string): Promise<Mesh>;
        };
        
        console: {
            log(...args: any[]): void;
            warn(...args: any[]): void;
            error(...args: any[]): void;
        };
    };
}
```

### Security Implementation

```rust
// Restricted global object setup
fn setup_restricted_globals(scope: &mut v8::HandleScope) -> v8::Local<v8::Object> {
    let global = v8::Object::new(scope);
    
    // Only expose allowed APIs based on permissions
    if permissions.has_capability("console_write") {
        setup_console_api(scope, global);
    }
    
    if permissions.has_capability("entity_read") {
        setup_world_api(scope, global);
    }
    
    // Block dangerous globals
    // No require, process, Buffer, etc.
    
    global
}
```

## Risk Assessment and Mitigation

### Technical Risks

1. **Performance Regression**
   - *Risk*: JavaScript slower than Lua
   - *Mitigation*: V8 JIT compilation, ahead-of-time optimization
   - *Fallback*: QuickJS for better embedding performance

2. **Memory Usage**
   - *Risk*: V8 has higher memory overhead
   - *Mitigation*: Isolate pooling, resource limits, GC tuning

3. **Compilation Overhead**
   - *Risk*: TypeScript compilation adds latency
   - *Mitigation*: Incremental compilation, caching, SWC performance

### Integration Risks

1. **API Compatibility**
   - *Risk*: Behavior differences between Lua and TypeScript APIs
   - *Mitigation*: Comprehensive test suite, gradual migration

2. **Third-Party Dependencies**
   - *Risk*: npm package security and bloat
   - *Mitigation*: Curated package allowlist, security scanning

### Migration Risks

1. **Existing Script Compatibility**
   - *Risk*: Breaking changes for existing Lua scripts
   - *Mitigation*: Automated transpilation tool, parallel runtime support

2. **Developer Adoption**
   - *Risk*: Learning curve for TypeScript
   - *Mitigation*: Comprehensive documentation, examples, gradual transition

## Success Metrics

### Performance Targets
- Script execution performance within 20% of Lua
- Hot reload time under 500ms for typical scripts
- Memory usage increase under 50%

### Developer Experience
- Type checking coverage >90%
- API documentation auto-generated from types
- Zero-config setup for new projects
- IDE integration with full IntelliSense

### Ecosystem Integration
- Support for major npm packages (limited allowlist)
- Integration with popular development tools
- Compatibility with existing editor workflows

## Timeline Summary

| Phase | Duration | Key Deliverable |
|-------|----------|----------------|
| 1 | 4-6 weeks | Basic TypeScript runtime |
| 2 | 3-4 weeks | Complete API bindings |
| 3 | 2-3 weeks | ECS integration |
| 4 | 3-4 weeks | Development experience |
| 5 | 2-3 weeks | Performance optimization |
| 6 | 2-3 weeks | Migration tools |

**Total Timeline: 16-23 weeks (4-6 months)**

## Resource Requirements

### Development Team
- 1-2 Rust developers for core runtime
- 1 TypeScript expert for API design
- 1 DevOps engineer for tooling and build systems

### Infrastructure
- CI/CD pipeline updates
- Additional testing infrastructure
- Documentation generation pipeline

## Next Steps

1. **Technical Spike**: 2-week proof of concept
   - Basic V8 integration
   - Simple TypeScript compilation
   - Performance baseline measurement

2. **Architecture Review**: Team review of detailed implementation plan

3. **Gradual Implementation**: Begin Phase 1 development with parallel Lua support

4. **Community Feedback**: Early access program for TypeScript scripting

This migration represents a significant advancement in the developer experience while maintaining the robust security and performance characteristics of the current Lua implementation.