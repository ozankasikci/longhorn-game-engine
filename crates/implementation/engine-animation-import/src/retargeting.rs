use crate::{AnimationData, AnimationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneMapping {
    pub source_bone: String,
    pub target_bone: String,
}

pub struct AnimationRetargeter;

impl AnimationRetargeter {
    pub fn new() -> Self {
        Self
    }

    pub fn retarget(
        &self,
        animation: &mut AnimationData,
        bone_mappings: &[BoneMapping],
    ) -> Result<(), AnimationError> {
        // Create a mapping lookup
        let mapping_lookup: std::collections::HashMap<_, _> = bone_mappings
            .iter()
            .map(|m| (m.source_bone.as_str(), m.target_bone.as_str()))
            .collect();

        // Update all channel target nodes
        for channel in &mut animation.channels {
            if let Some(target_bone) = mapping_lookup.get(channel.target_node.as_str()) {
                channel.target_node = target_bone.to_string();
            }
        }

        Ok(())
    }
}
