use serde::de::DeserializeOwned;
use std::io;

/// Load and deserialize JSON data from bytes
pub fn load_json<T: DeserializeOwned>(bytes: &[u8]) -> io::Result<T> {
    serde_json::from_slice(bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_load_json_valid() {
        let json = r#"{"name": "test", "value": 42}"#;
        let data: TestData = load_json(json.as_bytes()).unwrap();

        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    #[test]
    fn test_load_json_invalid() {
        let invalid_json = r#"{"name": "test", "value": }"#;
        let result: io::Result<TestData> = load_json(invalid_json.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_json_wrong_type() {
        let json = r#"{"name": "test", "value": "not a number"}"#;
        let result: io::Result<TestData> = load_json(json.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_json_array() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Item {
            id: u32,
        }

        let json = r#"[{"id": 1}, {"id": 2}, {"id": 3}]"#;
        let items: Vec<Item> = load_json(json.as_bytes()).unwrap();

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].id, 1);
        assert_eq!(items[2].id, 3);
    }
}
