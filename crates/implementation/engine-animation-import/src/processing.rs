use crate::{AnimationData, AnimationError, Channel, InterpolationType, Keyframe};

pub struct KeyframeOptimizer;

impl Default for KeyframeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyframeOptimizer {
    pub fn new() -> Self {
        Self
    }

    pub fn optimize(&self, channel: &mut Channel, tolerance: f32) -> Result<(), AnimationError> {
        if channel.keyframes.len() < 3 {
            return Ok(()); // Nothing to optimize
        }

        let mut optimized_keyframes = vec![channel.keyframes[0].clone()];
        let mut i = 1;

        while i < channel.keyframes.len() - 1 {
            let prev = &channel.keyframes[i - 1];
            let curr = &channel.keyframes[i];
            let next = &channel.keyframes[i + 1];

            // Check if current keyframe is on the linear path between prev and next
            if self.is_redundant(prev, curr, next, tolerance) {
                // Skip this keyframe
                i += 1;
                continue;
            }

            optimized_keyframes.push(curr.clone());
            i += 1;
        }

        // Always keep the last keyframe
        optimized_keyframes.push(channel.keyframes.last().unwrap().clone());

        channel.keyframes = optimized_keyframes;
        Ok(())
    }

    fn is_redundant(
        &self,
        prev: &Keyframe,
        curr: &Keyframe,
        next: &Keyframe,
        tolerance: f32,
    ) -> bool {
        // Only check for linear interpolation
        if curr.interpolation != InterpolationType::Linear {
            return false;
        }

        // Calculate interpolated value at current time
        let t = (curr.time - prev.time) / (next.time - prev.time);

        for i in 0..curr.value.len() {
            let interpolated = prev.value[i] + (next.value[i] - prev.value[i]) * t;
            if (interpolated - curr.value[i]).abs() > tolerance {
                return false;
            }
        }

        true
    }
}

pub struct FpsConverter;

impl Default for FpsConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn convert(
        &self,
        _animation: &mut AnimationData,
        source_fps: f32,
        target_fps: f32,
    ) -> Result<(), AnimationError> {
        if (source_fps - target_fps).abs() < 0.001 {
            return Ok(()); // No conversion needed
        }

        // For simplicity, we don't resample keyframes in this implementation
        // In a real implementation, you would resample keyframes at the target FPS
        // Time values remain the same as they're in seconds, not frames

        Ok(())
    }
}
