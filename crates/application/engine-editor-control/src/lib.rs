//! Editor Control System - Remote control interface for the Longhorn Engine Editor
//! 
//! This crate provides a way to programmatically control the editor without UI interaction.
//! It enables automated testing of script changes, scene inspection, and other editor operations.

pub mod commands;
pub mod server;
pub mod client;
pub mod types;

pub use commands::*;
pub use server::*;
pub use client::*;
pub use types::*;