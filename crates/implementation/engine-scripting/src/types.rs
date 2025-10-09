//! Script type definitions

use serde::{Deserialize, Serialize};

/// Script identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptId(pub u64);

/// Script type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    Lua,
    JavaScript,
    TypeScript,
    Python,
    Wasm,
}

/// Script metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub id: ScriptId,
    pub script_type: ScriptType,
    pub path: String,
    pub entry_point: Option<String>,
}
