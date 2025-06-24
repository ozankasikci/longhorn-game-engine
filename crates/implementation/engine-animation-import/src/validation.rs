use crate::{AnimationData, AnimationError};

pub struct AnimationValidator;

impl Default for AnimationValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, animation: &AnimationData) -> Result<(), AnimationError> {
        // Check animation has a name
        if animation.name.is_empty() {
            return Err(AnimationError::ValidationError(
                "Animation must have a name".to_string(),
            ));
        }

        // Check duration is positive
        if animation.duration_seconds <= 0.0 {
            return Err(AnimationError::ValidationError(
                "Animation duration must be positive".to_string(),
            ));
        }

        // Validate each channel
        for (i, channel) in animation.channels.iter().enumerate() {
            // Check channel has keyframes
            if channel.keyframes.is_empty() {
                return Err(AnimationError::ValidationError(format!(
                    "Channel {} has no keyframes",
                    i
                )));
            }

            // Check keyframes are in chronological order
            let mut last_time = -1.0;
            for (j, keyframe) in channel.keyframes.iter().enumerate() {
                if keyframe.time <= last_time {
                    return Err(AnimationError::ValidationError(format!(
                        "Channel {} keyframe {} is out of order",
                        i, j
                    )));
                }
                last_time = keyframe.time;

                // Check keyframe time is within animation duration
                if keyframe.time > animation.duration_seconds {
                    return Err(AnimationError::ValidationError(format!(
                        "Channel {} keyframe {} time exceeds animation duration",
                        i, j
                    )));
                }

                // Check value count based on property type
                let expected_values = match &channel.property {
                    crate::PropertyType::Position => 3,
                    crate::PropertyType::Rotation => 4, // Quaternion
                    crate::PropertyType::Scale => 3,
                    crate::PropertyType::BlendShape(_) => 1,
                };

                if keyframe.value.len() != expected_values {
                    return Err(AnimationError::ValidationError(format!(
                        "Channel {} keyframe {} has wrong number of values",
                        i, j
                    )));
                }
            }
        }

        Ok(())
    }
}
