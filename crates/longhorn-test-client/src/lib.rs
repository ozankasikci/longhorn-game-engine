//! Test client for Longhorn Editor remote control.
//!
//! Provides a typed Rust API for controlling the editor via its Unix socket interface.
//!
//! # Example
//! ```ignore
//! use longhorn_test_client::{EditorClient, EditorError};
//!
//! let client = EditorClient::connect("/tmp/longhorn-editor.sock")?;
//! client.ping()?;
//!
//! let state = client.get_state()?;
//! println!("Mode: {}, Entity count: {}", state.mode, state.entity_count);
//! ```

mod client;
mod error;
mod harness;
mod responses;

pub use client::EditorClient;
pub use error::EditorError;
pub use harness::TestHarness;
pub use responses::*;
