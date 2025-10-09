//! Code generation for TypeScript definitions and documentation

pub mod typescript_gen;
pub mod docs_gen;

pub use typescript_gen::TypeScriptGenerator;
pub use docs_gen::DocumentationGenerator;