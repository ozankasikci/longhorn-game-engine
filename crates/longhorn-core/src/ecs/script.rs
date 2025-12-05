// crates/longhorn-core/src/ecs/script.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Value types for script properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScriptValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Vec2 { x: f64, y: f64 },
}

impl ScriptValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ScriptValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            ScriptValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ScriptValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Script component - attached to entities to run TypeScript code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    /// Path to the script file (relative to scripts/ folder)
    pub path: String,
    /// Instance properties (overrides class defaults)
    pub properties: HashMap<String, ScriptValue>,
    /// Whether this script is enabled
    pub enabled: bool,
}

impl Script {
    /// Create a new script component
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            properties: HashMap::new(),
            enabled: true,
        }
    }

    /// Create a script with properties
    pub fn with_properties(path: impl Into<String>, properties: HashMap<String, ScriptValue>) -> Self {
        Self {
            path: path.into(),
            properties,
            enabled: true,
        }
    }

    /// Get a property value
    pub fn get_property(&self, name: &str) -> Option<&ScriptValue> {
        self.properties.get(name)
    }

    /// Set a property value
    pub fn set_property(&mut self, name: impl Into<String>, value: ScriptValue) {
        self.properties.insert(name.into(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_new() {
        let script = Script::new("PlayerController.ts");
        assert_eq!(script.path, "PlayerController.ts");
        assert!(script.properties.is_empty());
        assert!(script.enabled);
    }

    #[test]
    fn test_script_with_properties() {
        let mut props = HashMap::new();
        props.insert("speed".to_string(), ScriptValue::Number(5.0));

        let script = Script::with_properties("PlayerController.ts", props);
        assert_eq!(script.get_property("speed"), Some(&ScriptValue::Number(5.0)));
    }

    #[test]
    fn test_script_value_accessors() {
        assert_eq!(ScriptValue::Number(5.0).as_number(), Some(5.0));
        assert_eq!(ScriptValue::String("test".into()).as_string(), Some("test"));
        assert_eq!(ScriptValue::Boolean(true).as_bool(), Some(true));
    }

    #[test]
    fn test_script_value_serialization() {
        let value = ScriptValue::Number(42.0);
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, "42.0");

        let parsed: ScriptValue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, value);
    }
}
