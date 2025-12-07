//! Integration tests for the Longhorn Editor.
//!
//! These tests use the EditorClient to communicate with a running editor instance.
//! Run with: cargo test --test integration -- --ignored
//!
//! Note: Tests marked with #[ignore] require a running editor instance.

mod state_tests;
mod entity_tests;
