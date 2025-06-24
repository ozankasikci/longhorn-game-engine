use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSettings {
    pub generate_mipmaps: bool,
    pub optimize_meshes: bool,
    pub max_texture_size: u32,
    pub import_materials: bool,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            generate_mipmaps: true,
            optimize_meshes: true,
            max_texture_size: 4096,
            import_materials: true,
            custom_settings: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportContext {
    settings: ImportSettings,
}

impl ImportContext {
    pub fn new(settings: ImportSettings) -> Self {
        Self { settings }
    }

    pub fn settings(&self) -> &ImportSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut ImportSettings {
        &mut self.settings
    }

    pub fn get_custom_setting<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.settings
            .custom_settings
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set_custom_setting<T: Serialize>(
        &mut self,
        key: String,
        value: T,
    ) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.settings.custom_settings.insert(key, json_value);
        Ok(())
    }
}
