//! Script bindings module

mod entity_handle;
pub use entity_handle::{EntityHandle, ComponentRef, ComponentMut};

mod legacy;
pub use legacy::{ScriptBindings, create_safe_bindings};

#[cfg(test)]
mod entity_handle_tests;