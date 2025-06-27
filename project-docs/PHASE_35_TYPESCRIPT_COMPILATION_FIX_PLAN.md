# Phase 35: TypeScript Compilation Fix Plan

## Overview

Fix the TypeScript script execution issue where ES6 module syntax (`export class`) is incompatible with V8's script execution context. Replace the current regex-based workaround with proper SWC compiler configuration.

## Problem Statement

**Current Issue**: TypeScript scripts compile to ES6 module syntax but fail to execute in V8 with error:
```
"<unknown>:0: Uncaught SyntaxError: Unexpected token 'export'"
```

**Root Cause**: SWC TypeScript compiler outputs ES6 modules by default, but V8's `isolate.run_script()` expects regular JavaScript without ES6 module syntax.

**Current Workaround**: Regex-based string replacement (incomplete and fragile)

## Research Summary

### SWC Configuration Options
- SWC can output CommonJS, UMD, or IIFE formats instead of ES6 modules
- Configuration via module type settings in compilation options
- Proper compiler-based solution vs runtime string manipulation

### V8 Embedded JavaScript Compatibility  
- V8 embedded in Rust works best with IIFE/CommonJS patterns
- ES6 modules require complex module resolution setup
- Script execution context doesn't support `export` statements

## Solution Plan

### Phase 1: SWC Configuration Research ✅ PLANNED
**Tasks:**
- [ ] Investigate current SWC configuration in TypeScript runtime
- [ ] Identify where module type is set (or defaulted)
- [ ] Document current compilation pipeline

**Files to examine:**
- `crates/implementation/engine-scripting/src/typescript_script_system.rs`
- SWC integration points in TypeScript runtime

### Phase 2: Compiler Configuration Fix ✅ PLANNED
**Tasks:**
- [ ] Configure SWC to output IIFE/UMD instead of ES6 modules
- [ ] Update compilation options in Rust code:
  ```rust
  let config = swc::config::Config {
      module: Some(ModuleConfig {
          type_: ModuleType::Umd, // or CommonJs
          ..Default::default()
      }),
      ..Default::default()
  };
  ```
- [ ] Remove regex-based export conversion code entirely

### Phase 3: Testing & Validation ✅ PLANNED
**Tasks:**
- [ ] Test TypeScript hello world script execution
- [ ] Verify console output appears in game engine console  
- [ ] Test with more complex TypeScript examples
- [ ] Validate all export types work (class, function, const, default)

## Expected Benefits

1. **Robust Solution**: Handles ALL export types automatically
2. **Performance**: No runtime regex processing overhead
3. **Maintainability**: Standard compiler configuration vs custom string manipulation
4. **Compatibility**: V8-compatible JavaScript output from compilation

## Success Criteria

- [ ] TypeScript scripts execute without syntax errors
- [ ] Console output appears in game engine console
- [ ] No runtime string manipulation needed
- [ ] All TypeScript examples work correctly

## Timeline

- **Phase 1**: Research current configuration (1-2 hours)
- **Phase 2**: Implement SWC configuration fix (2-3 hours) 
- **Phase 3**: Testing and validation (1-2 hours)

**Total estimated time**: 4-7 hours

## Dependencies

- SWC TypeScript compiler integration
- V8 JavaScript runtime in Rust
- TypeScript script system architecture

## Risks & Mitigation

**Risk**: SWC configuration changes break existing functionality
**Mitigation**: Test thoroughly with existing TypeScript examples

**Risk**: IIFE/UMD output format incompatible with current API injection
**Mitigation**: Validate globalThis API access works with new format

## Related Files

- `crates/implementation/engine-scripting/src/typescript_script_system.rs`
- `assets/scripts/typescript_hello_world.ts`
- `project-docs/PHASE_34_TYPESCRIPT_EXECUTION_SYSTEM_PROGRESS.md`