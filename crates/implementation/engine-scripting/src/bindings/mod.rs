//! Script bindings module

mod entity_handle;
pub use entity_handle::{EntityHandle, ComponentRef, ComponentMut};


#[cfg(test)]
mod entity_handle_tests;