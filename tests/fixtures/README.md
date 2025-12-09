# Test Fixtures

This directory contains minimal game projects used for integration testing.

## empty_project

A minimal valid Longhorn game project with no scripts or assets.

**Purpose:**
- Test engine/editor can handle projects with minimal configuration
- Validate project structure requirements
- Test error handling when entry scripts are missing
- Serve as a baseline for "what is the absolute minimum valid project"

**Structure:**
```
empty_project/
├── game.json       # Minimal game configuration (no entry point)
└── assets.json     # Empty assets registry
```

**Used by:**
- `tests/integration_tests.rs` - Project structure validation tests

**When to use:**
- Testing project loading edge cases
- Validating minimum requirements for a valid project
- Testing editor/engine behavior with incomplete projects
